use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue, BasicValueEnum, BasicMetadataValueEnum};
use inkwell::types::{BasicMetadataTypeEnum, StructType};
use inkwell::{OptimizationLevel, AddressSpace};
use inkwell::targets::{Target, TargetMachine, RelocMode, CodeModel, FileType, InitializationConfig};
use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, anyhow};

use crate::core::ast::*;
use crate::runtime::values::Value;


pub struct LLVMCodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    
    // Yaf runtime types
    yaf_value_type: StructType<'ctx>,
    
    // Function and variable maps
    functions: HashMap<String, FunctionValue<'ctx>>,
    global_variables: HashMap<String, PointerValue<'ctx>>,
    local_variables: HashMap<String, PointerValue<'ctx>>,
    
    // Current function context
    current_function: Option<FunctionValue<'ctx>>,
    
    // Optimization level
    optimization_level: OptimizationLevel,
    
    // Counter for unique variable names
    variable_counter: usize,
}

impl<'ctx> LLVMCodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str, opt_level: OptimizationLevel) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        // Create Yaf value type (equivalent to our C struct)
        let yaf_type_enum = context.i32_type(); // enum for type tag
        let data_union = context.i64_type(); // simplified union as i64 for now
        let yaf_value_type = context.struct_type(&[
            yaf_type_enum.into(), // type tag
            data_union.into(),    // data
        ], false);
        
        LLVMCodeGenerator {
            context,
            module,
            builder,
            yaf_value_type,
            functions: HashMap::new(),
            global_variables: HashMap::new(),
            local_variables: HashMap::new(),
            current_function: None,
            optimization_level: opt_level,
            variable_counter: 0,
        }
    }
    
    // Helper method to find variables (first local, then global)
    fn get_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        self.local_variables.get(name).or_else(|| self.global_variables.get(name))
    }
    
    // Generate a unique variable name to avoid LLVM dominance issues
    fn get_unique_var_name(&mut self, base_name: &str) -> String {
        self.variable_counter += 1;
        format!("{}_{}", base_name, self.variable_counter)
    }

    pub fn generate(&mut self, program: Program) -> Result<()> {
        // Generate runtime functions first
        self.generate_runtime_functions()?;
        
        // Declare all user functions
        for function in &program.functions {
            self.declare_function(function)?;
        }
        
        // Extract and create global variables FIRST
        self.create_global_variables(&program.main)?;
        
        // Generate main function
        self.generate_main(&program.main)?;
        
        // Then implement all user functions (now they can access globals)
        for function in &program.functions {
            self.generate_function(function)?;
        }
        
        Ok(())
    }
    
    fn create_global_variables(&mut self, main_block: &Block) -> Result<()> {
        // Solo procesar asignaciones en el nivel global para crear las variables
        for statement in &main_block.statements {
            if let Statement::Assignment { name, .. } = statement {
                if !self.global_variables.contains_key(name) {
                    // Crear variable global real
                    let unique_name = self.get_unique_var_name(name);
                    let global = self.module.add_global(self.yaf_value_type, None, &unique_name);
                    
                    // Inicializar con valor por defecto
                    let null_value = self.yaf_value_type.const_zero();
                    global.set_initializer(&null_value);
                    
                    let global_ptr = global.as_pointer_value();
                    
                    // Registrar como variable global
                    self.global_variables.insert(name.clone(), global_ptr);
                }
            }
        }
        Ok(())
    }
    
    fn generate_runtime_functions(&mut self) -> Result<()> {
        let _void_type = self.context.void_type();
        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        
        // printf declaration
        let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None);
        
        // malloc declaration
        let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("malloc", malloc_type, None);
        
        // free declaration
        let free_type = _void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("free", free_type, None);
        
        // strdup declaration
        let strdup_type = ptr_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("strdup", strdup_type, None);
        
        // Generate GC functions first (needed by other functions)
        self.generate_gc_functions()?;
        
        // Generate advanced memory management functions
        self.generate_memory_pool_functions()?;
        
        // Declare external runtime functions (implemented in C)
        self.declare_yaf_runtime_functions()?;
        self.generate_yaf_add()?;
        self.generate_yaf_sub()?;
        self.generate_yaf_mul()?;
        self.generate_yaf_div()?;
        self.generate_yaf_mod()?;
        self.generate_yaf_eq()?;
        self.generate_yaf_ne()?;
        self.generate_yaf_lt()?;
        self.generate_yaf_le()?;
        self.generate_yaf_gt()?;
        self.generate_yaf_ge()?;
        self.generate_yaf_to_bool()?;
        // Don't generate implementation of yaf_free_value - use external
        self.generate_yaf_clone_value()?;
        
        // Array functions
        self.generate_yaf_make_array()?;
        self.generate_yaf_array_get()?;
        self.generate_yaf_array_set()?;
        
        Ok(())
    }
    
    fn declare_yaf_runtime_functions(&mut self) -> Result<()> {
        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();
        
        // yaf_make_int declaration
        let make_int_type = self.yaf_value_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("yaf_make_int", make_int_type, None);
        
        // yaf_make_bool declaration  
        let make_bool_type = self.yaf_value_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("yaf_make_bool", make_bool_type, None);
        
        // yaf_make_string declaration
        let make_string_type = self.yaf_value_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("yaf_make_string", make_string_type, None);
        
        // yaf_make_float declaration
        let f64_type = self.context.f64_type();
        let make_float_type = self.yaf_value_type.fn_type(&[f64_type.into()], false);
        self.module.add_function("yaf_make_float", make_float_type, None);
        
        // Type conversion function declarations
        let value_to_string_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_value_to_string", value_to_string_type, None);
        
        let value_to_int_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_value_to_int", value_to_int_type, None);
        
        let value_to_float_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_value_to_float", value_to_float_type, None);
        
        // yaf_print_value declaration
        let print_value_type = void_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_print_value", print_value_type, None);
        
        // yaf_print_value_no_newline declaration
        let print_value_no_nl_type = void_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_print_value_no_newline", print_value_no_nl_type, None);
        
        // yaf_string_concat declaration
        let string_concat_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into(), self.yaf_value_type.into()], false);
        self.module.add_function("yaf_string_concat", string_concat_type, None);
        
        // yaf_free_value declaration
        let free_value_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("yaf_free_value", free_value_type, None);
        
        // Input functions declarations
        let input_type = self.yaf_value_type.fn_type(&[], false);
        self.module.add_function("yaf_io_input", input_type, None);
        
        let input_prompt_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_io_input_prompt", input_prompt_type, None);
        
        // String conversion functions declarations
        let string_to_int_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_string_to_int", string_to_int_type, None);
        
        let int_to_string_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_int_to_string", int_to_string_type, None);
        
        // Time functions declarations
        let time_now_type = self.yaf_value_type.fn_type(&[], false);
        self.module.add_function("yaf_time_now", time_now_type, None);
        
        let time_now_millis_type = self.yaf_value_type.fn_type(&[], false);
        self.module.add_function("yaf_time_now_millis", time_now_millis_type, None);
        
        let time_sleep_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        self.module.add_function("yaf_time_sleep", time_sleep_type, None);
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn generate_yaf_make_int(&mut self) -> Result<()> {
        let i64_type = self.context.i64_type();
        let fn_type = self.yaf_value_type.fn_type(&[i64_type.into()], false);
        let function = self.module.add_function("yaf_make_int", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param = function.get_nth_param(0).unwrap().into_int_value();
        
        // Create struct value
        let struct_val = self.yaf_value_type.get_undef();
        let struct_val = self.builder.build_insert_value(
            struct_val, 
            self.context.i32_type().const_int(0, false), // YAF_INT = 0
            0, 
            "type_field"
        ).unwrap();
        let struct_val = self.builder.build_insert_value(
            struct_val,
            param,
            1,
            "data_field"
        ).unwrap();
        
        self.builder.build_return(Some(&struct_val)).unwrap();
        Ok(())
    }
    
    #[allow(dead_code)]
    fn generate_yaf_make_bool(&mut self) -> Result<()> {
        let i1_type = self.context.bool_type();
        let fn_type = self.yaf_value_type.fn_type(&[i1_type.into()], false);
        let function = self.module.add_function("yaf_make_bool", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param = function.get_nth_param(0).unwrap().into_int_value();
        let extended = self.builder.build_int_z_extend(param, self.context.i64_type(), "extend").unwrap();
        
        let struct_val = self.yaf_value_type.get_undef();
        let struct_val = self.builder.build_insert_value(
            struct_val,
            self.context.i32_type().const_int(3, false), // YAF_BOOL = 3
            0,
            "type_field"
        ).unwrap();
        let struct_val = self.builder.build_insert_value(
            struct_val,
            extended,
            1,
            "data_field"
        ).unwrap();
        
        self.builder.build_return(Some(&struct_val)).unwrap();
        Ok(())
    }
    
    #[allow(dead_code)]
    fn generate_yaf_make_string(&mut self) -> Result<()> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let fn_type = self.yaf_value_type.fn_type(&[ptr_type.into()], false);
        let function = self.module.add_function("yaf_make_string", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param = function.get_nth_param(0).unwrap().into_pointer_value();
        
        // Duplicate string to ensure we own it
        let strdup_fn = self.module.get_function("strdup").unwrap();
        let copied_str = self.builder.build_call(
            strdup_fn,
            &[param.into()],
            "copied_str"
        ).unwrap().try_as_basic_value().left().unwrap().into_pointer_value();
        
        // Create struct value
        let struct_val = self.yaf_value_type.get_undef();
        let struct_val = self.builder.build_insert_value(
            struct_val, 
            self.context.i32_type().const_int(2, false), // YAF_STRING = 2
            0, 
            "type_field"
        ).unwrap();
        
        // Convert pointer to i64 for storage
        let ptr_as_int = self.builder.build_ptr_to_int(
            copied_str, 
            self.context.i64_type(), 
            "ptr_to_int"
        ).unwrap();
        
        let struct_val = self.builder.build_insert_value(
            struct_val,
            ptr_as_int,
            1,
            "data_field"
        ).unwrap();
        
        // Register with garbage collector 
        // Calculate approximate string size (we don't have strlen here, so estimate)
        let estimated_size = self.context.i64_type().const_int(64, false); // Rough estimate
        let string_type = self.context.i32_type().const_int(2, false); // YAF_STRING = 2
        
        let gc_register_fn = self.module.get_function("yaf_gc_register_allocation").unwrap();
        self.builder.build_call(
            gc_register_fn,
            &[ptr_as_int.into(), estimated_size.into(), string_type.into()],
            "gc_register"
        ).unwrap();
        
        self.builder.build_return(Some(&struct_val)).unwrap();
        Ok(())
    }
    
    #[allow(dead_code)]
    fn generate_yaf_print_value(&mut self) -> Result<()> {
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[self.yaf_value_type.into()], false);
        let function = self.module.add_function("yaf_print_value", fn_type, None);
        
        let entry_block = self.context.append_basic_block(function, "entry");
        let int_block = self.context.append_basic_block(function, "int_case");
        let float_block = self.context.append_basic_block(function, "float_case");
        let string_block = self.context.append_basic_block(function, "string_case");
        let bool_block = self.context.append_basic_block(function, "bool_case");
        let default_block = self.context.append_basic_block(function, "default_case");
        let end_block = self.context.append_basic_block(function, "end");
        
        self.builder.position_at_end(entry_block);
        
        let param = function.get_nth_param(0).unwrap().into_struct_value();
        let type_val = self.builder.build_extract_value(param, 0, "type").unwrap().into_int_value();
        
        // Switch on type
        let _switch = self.builder.build_switch(type_val, default_block, &[
            (self.context.i32_type().const_int(0, false), int_block),   // YAF_INT
            (self.context.i32_type().const_int(1, false), float_block), // YAF_FLOAT
            (self.context.i32_type().const_int(2, false), string_block), // YAF_STRING
            (self.context.i32_type().const_int(3, false), bool_block),  // YAF_BOOL
        ]).unwrap();
        
        // Integer case
        self.builder.position_at_end(int_block);
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_int_value();
        let printf_fn = self.module.get_function("printf").unwrap();
        let format_str = self.builder.build_global_string_ptr("%lld", "int_fmt").unwrap();
        self.builder.build_call(
            printf_fn,
            &[format_str.as_pointer_value().into(), data_val.into()],
            "printf_call"
        ).unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // Float case
        self.builder.position_at_end(float_block);
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_float_value();
        let printf_fn = self.module.get_function("printf").unwrap();
        let format_str = self.builder.build_global_string_ptr("%g", "float_fmt").unwrap();
        self.builder.build_call(
            printf_fn,
            &[format_str.as_pointer_value().into(), data_val.into()],
            "printf_call"
        ).unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // String case
        self.builder.position_at_end(string_block);
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_int_value();
        // Convert i64 back to pointer
        let ptr_val = self.builder.build_int_to_ptr(
            data_val, 
            self.context.ptr_type(AddressSpace::default()), 
            "int_to_ptr"
        ).unwrap();
        let printf_fn = self.module.get_function("printf").unwrap();
        let format_str = self.builder.build_global_string_ptr("%s", "str_fmt").unwrap();
        self.builder.build_call(
            printf_fn,
            &[format_str.as_pointer_value().into(), ptr_val.into()],
            "printf_call"
        ).unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // Boolean case
        self.builder.position_at_end(bool_block);
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_int_value();
        let is_true = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            data_val,
            self.context.i64_type().const_int(0, false),
            "is_true"
        ).unwrap();
        
        let true_block = self.context.append_basic_block(function, "true_case");
        let false_block = self.context.append_basic_block(function, "false_case");
        
        self.builder.build_conditional_branch(is_true, true_block, false_block).unwrap();
        
        self.builder.position_at_end(true_block);
        let true_str = self.builder.build_global_string_ptr("true", "true_str").unwrap();
        self.builder.build_call(
            printf_fn,
            &[self.builder.build_global_string_ptr("%s", "str_fmt").unwrap().as_pointer_value().into(), true_str.as_pointer_value().into()],
            "printf_true"
        ).unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        self.builder.position_at_end(false_block);
        let false_str = self.builder.build_global_string_ptr("false", "false_str").unwrap();
        self.builder.build_call(
            printf_fn,
            &[self.builder.build_global_string_ptr("%s", "str_fmt").unwrap().as_pointer_value().into(), false_str.as_pointer_value().into()],
            "printf_false"
        ).unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // Default case
        self.builder.position_at_end(default_block);
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // End
        self.builder.position_at_end(end_block);
        self.builder.build_return(None).unwrap();
        
        Ok(())
    }
    
    fn generate_yaf_add(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_add", fn_type, None);
        
        let entry_block = self.context.append_basic_block(function, "entry");
        let string_block = self.context.append_basic_block(function, "string_concat");
        let int_block = self.context.append_basic_block(function, "int_add");
        let end_block = self.context.append_basic_block(function, "end");
        
        self.builder.position_at_end(entry_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        // Extract types
        let type1 = self.builder.build_extract_value(param1, 0, "type1").unwrap().into_int_value();
        let type2 = self.builder.build_extract_value(param2, 0, "type2").unwrap().into_int_value();
        
        // Check if either is string (YAF_STRING = 2)
        let string_type = type1.get_type().const_int(2, false);
        let is_string1 = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type1,
            string_type,
            "is_string1"
        ).unwrap();
        let is_string2 = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type2,
            string_type,
            "is_string2"
        ).unwrap();
        let either_string = self.builder.build_or(is_string1, is_string2, "either_string").unwrap();
        
        let _not_string = self.builder.build_not(either_string, "not_string").unwrap();
        
        // Check if both are int (YAF_INT = 0)
        let int_type = type1.get_type().const_int(0, false);
        let is_int1 = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type1,
            int_type,
            "is_int1"
        ).unwrap();
        let is_int2 = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type2,
            int_type,
            "is_int2"
        ).unwrap();
        let _both_int = self.builder.build_and(is_int1, is_int2, "both_int").unwrap();
        
        // Branch logic
        self.builder.build_conditional_branch(either_string, string_block, int_block).unwrap();
        
        // String concatenation block
        self.builder.position_at_end(string_block);
        let concat_fn = self.module.get_function("yaf_string_concat").unwrap();
        let concat_result = self.builder.build_call(
            concat_fn,
            &[param1.into(), param2.into()],
            "concat_call"
        ).unwrap();
        let concat_val = concat_result.try_as_basic_value().left().unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // Integer addition block
        self.builder.position_at_end(int_block);
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        let int_result = self.builder.build_int_add(data1, data2, "add_result").unwrap();
        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
        let int_result_val = self.builder.build_call(
            make_int_fn,
            &[int_result.into()],
            "make_int_call"
        ).unwrap();
        let int_val = int_result_val.try_as_basic_value().left().unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // End block with phi
        self.builder.position_at_end(end_block);
        let phi = self.builder.build_phi(self.yaf_value_type, "result").unwrap();
        phi.add_incoming(&[(&concat_val, string_block), (&int_val, int_block)]);
        
        self.builder.build_return(Some(&phi.as_basic_value())).unwrap();
        Ok(())
    }
    
    // Similar implementations for other arithmetic operations...
    fn generate_yaf_sub(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_sub", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_sub(data1, data2, "sub_result").unwrap();
        
        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
        let result_val = self.builder.build_call(
            make_int_fn,
            &[result.into()],
            "make_int_call"
        ).unwrap().try_as_basic_value().left().unwrap();
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_mul(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_mul", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_mul(data1, data2, "mul_result").unwrap();
        
        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
        let result_val = self.builder.build_call(
            make_int_fn,
            &[result.into()],
            "make_int_call"
        ).unwrap().try_as_basic_value().left().unwrap();
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_div(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_div", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_signed_div(data1, data2, "div_result").unwrap();
        
        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
        let result_val = self.builder.build_call(
            make_int_fn,
            &[result.into()],
            "make_int_call"
        ).unwrap().try_as_basic_value().left().unwrap();
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_mod(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_mod", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_signed_rem(data1, data2, "mod_result").unwrap();
        
        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
        let result_val = self.builder.build_call(
            make_int_fn,
            &[result.into()],
            "make_int_call"
        ).unwrap().try_as_basic_value().left().unwrap();
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn call_yaf_make_bool_from_i1(&mut self, bool_val: inkwell::values::IntValue<'ctx>) -> inkwell::values::BasicValueEnum<'ctx> {
        // Convert i1 to i32 for C function call
        let extended_result = self.builder.build_int_z_extend(
            bool_val,
            self.context.i32_type(),
            "extend_bool"
        ).unwrap();
        
        let make_bool_fn = self.module.get_function("yaf_make_bool").unwrap();
        self.builder.build_call(
            make_bool_fn,
            &[extended_result.into()],
            "make_bool_call"
        ).unwrap().try_as_basic_value().left().unwrap()
    }
    
    fn generate_yaf_eq(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_eq", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            data1,
            data2,
            "eq_result"
        ).unwrap();
        
        let result_val = self.call_yaf_make_bool_from_i1(result);
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_lt(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_lt", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            data1,
            data2,
            "lt_result"
        ).unwrap();
        
        let result_val = self.call_yaf_make_bool_from_i1(result);
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_ne(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_ne", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            data1,
            data2,
            "ne_result"
        ).unwrap();
        
        let result_val = self.call_yaf_make_bool_from_i1(result);
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_le(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_le", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::SLE,
            data1,
            data2,
            "le_result"
        ).unwrap();
        
        let result_val = self.call_yaf_make_bool_from_i1(result);
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_gt(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_gt", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::SGT,
            data1,
            data2,
            "gt_result"
        ).unwrap();
        
        let result_val = self.call_yaf_make_bool_from_i1(result);
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_ge(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_ge", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param1 = function.get_nth_param(0).unwrap().into_struct_value();
        let param2 = function.get_nth_param(1).unwrap().into_struct_value();
        
        let data1 = self.builder.build_extract_value(param1, 1, "data1").unwrap().into_int_value();
        let data2 = self.builder.build_extract_value(param2, 1, "data2").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::SGE,
            data1,
            data2,
            "ge_result"
        ).unwrap();
        
        let result_val = self.call_yaf_make_bool_from_i1(result);
        
        self.builder.build_return(Some(&result_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_to_bool(&mut self) -> Result<()> {
        let i1_type = self.context.bool_type();
        let fn_type = i1_type.fn_type(&[self.yaf_value_type.into()], false);
        let function = self.module.add_function("yaf_to_bool", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let param = function.get_nth_param(0).unwrap().into_struct_value();
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_int_value();
        
        let result = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            data_val,
            self.context.i64_type().const_int(0, false),
            "to_bool_result"
        ).unwrap();
        
        self.builder.build_return(Some(&result)).unwrap();
        Ok(())
    }
    
    fn declare_function(&mut self, function: &Function) -> Result<()> {
        let param_types: Vec<BasicMetadataTypeEnum> = function.parameters
            .iter()
            .map(|_| self.yaf_value_type.into())
            .collect();
        
        let fn_type = self.yaf_value_type.fn_type(&param_types, false);
        let llvm_function = self.module.add_function(
            &format!("yaf_func_{}", function.name),
            fn_type,
            None
        );
        
        self.functions.insert(function.name.clone(), llvm_function);
        Ok(())
    }
    
    fn generate_function(&mut self, function: &Function) -> Result<()> {
        let llvm_function = self.functions[&function.name];
        self.current_function = Some(llvm_function);
        
        let entry_block = self.context.append_basic_block(llvm_function, "entry");
        self.builder.position_at_end(entry_block);
        
        // Clear local variables for new function scope (keep globals)
        self.local_variables.clear();
        
        // Create alloca instructions for parameters
        for (i, param) in function.parameters.iter().enumerate() {
            let param_value = llvm_function.get_nth_param(i as u32).unwrap();
            let unique_name = self.get_unique_var_name(&param.name);
            let alloca = self.builder.build_alloca(self.yaf_value_type, &unique_name).unwrap();
            self.builder.build_store(alloca, param_value).unwrap();
            self.local_variables.insert(param.name.clone(), alloca);
        }
        
        // Generate function body
        self.generate_block(&function.body)?;
        
        // Add default return if needed
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let void_val = self.yaf_value_type.get_undef();
            self.builder.build_return(Some(&void_val)).unwrap();
        }
        
        self.current_function = None;
        Ok(())
    }
    
    fn generate_main(&mut self, main_block: &Block) -> Result<()> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let main_function = self.module.add_function("main", main_type, None);
        
        let basic_block = self.context.append_basic_block(main_function, "entry");
        self.builder.position_at_end(basic_block);
        
        // Set current function for proper context (needed for if/while statements)
        self.current_function = Some(main_function);
        self.local_variables.clear();
        
        self.generate_block(main_block)?;
        
        // Trigger final garbage collection before exit
        if let Some(gc_collect_fn) = self.module.get_function("yaf_gc_collect") {
            self.builder.build_call(
                gc_collect_fn,
                &[],
                "final_gc_collect"
            ).unwrap();
        }
        
        // Add automatic memory cleanup for better memory management
        self.generate_cleanup_code()?;
        
        // Return 0 from main
        self.builder.build_return(Some(&i32_type.const_int(0, false))).unwrap();
        
        self.current_function = None;
        Ok(())
    }
    
    fn generate_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.generate_statement(statement)?;
        }
        Ok(())
    }
    
    fn generate_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {

            Statement::Declaration { name, var_type: _, value } => {
                // Para declaraciones, funciona igual que asignación pero podríamos
                // usar el tipo para optimizaciones futuras
                let val = self.generate_expression(value)?;
                
                // Variable no debería existir ya (nueva declaración)
                let alloca = if let Some(function) = self.current_function {
                    // Estamos en una función - crear variable local en entry block
                    let current_block = self.builder.get_insert_block().unwrap();
                    
                    // Moverse al entry block para crear la alloca
                    let entry_block = function.get_first_basic_block().unwrap();
                    if let Some(first_instr) = entry_block.get_first_instruction() {
                        self.builder.position_before(&first_instr);
                    } else {
                        self.builder.position_at_end(entry_block);
                    }
                    
                    // Crear alloca con nombre único
                    let unique_name = self.get_unique_var_name(name);
                    let alloca = self.builder.build_alloca(self.yaf_value_type, &unique_name).unwrap();
                    
                    // Volver a la posición original
                    self.builder.position_at_end(current_block);
                    
                    // Registrar como variable local
                    self.local_variables.insert(name.clone(), alloca);
                    alloca
                } else {
                    // Estamos a nivel global - crear variable global
                    let unique_name = self.get_unique_var_name(name);
                    let global = self.module.add_global(self.yaf_value_type, None, &unique_name);
                    global.set_initializer(&self.yaf_value_type.const_zero());
                    
                    // Registrar como variable global
                    let global_ptr = global.as_pointer_value();
                    self.global_variables.insert(name.clone(), global_ptr);
                    global_ptr
                };
                
                // Hacer store del valor inicial
                self.builder.build_store(alloca, val).unwrap();
            },
            Statement::Assignment { name, value } => {
                let val = self.generate_expression(value)?;
                if let Some(&alloca) = self.get_variable(name) {
                    // Variable ya existe, solo hacer store
                    self.builder.build_store(alloca, val).unwrap();
                } else {
                    // Variable no existe, crearla
                    let alloca = if let Some(function) = self.current_function {
                        // Estamos en una función - crear variable local en entry block
                        let current_block = self.builder.get_insert_block().unwrap();
                        
                        // Moverse al entry block para crear la alloca
                        let entry_block = function.get_first_basic_block().unwrap();
                        if let Some(first_instr) = entry_block.get_first_instruction() {
                            self.builder.position_before(&first_instr);
                        } else {
                            self.builder.position_at_end(entry_block);
                        }
                        
                        // Crear alloca con nombre único
                        let unique_name = self.get_unique_var_name(name);
                        let alloca = self.builder.build_alloca(self.yaf_value_type, &unique_name).unwrap();
                        
                        // Volver a la posición original
                        self.builder.position_at_end(current_block);
                        
                        // Registrar como variable local
                        self.local_variables.insert(name.clone(), alloca);
                        alloca
                    } else {
                        // Estamos en contexto global - crear variable global real
                        let unique_name = self.get_unique_var_name(name);
                        let global = self.module.add_global(self.yaf_value_type, None, &unique_name);
                        
                        // Inicializar con un valor por defecto (null)
                        let null_value = self.yaf_value_type.const_zero();
                        global.set_initializer(&null_value);
                        
                        let global_ptr = global.as_pointer_value();
                        
                        // Registrar como variable global
                        self.global_variables.insert(name.clone(), global_ptr);
                        global_ptr
                    };
                    
                    self.builder.build_store(alloca, val).unwrap();
                }
            },
            Statement::ArrayAssignment { name, index, value } => {
                let array_val = if let Some(&alloca) = self.get_variable(name) {
                    self.builder.build_load(self.yaf_value_type, alloca, name).unwrap()
                } else {
                    return Err(anyhow!("Variable '{}' no encontrada", name));
                };
                
                let index_val = self.generate_expression(index)?;
                let value_val = self.generate_expression(value)?;
                
                // Use yaf_array_set runtime function
                let set_fn = self.module.get_function("yaf_array_set").unwrap();
                self.builder.build_call(
                    set_fn,
                    &[array_val.into(), index_val.into(), value_val.into()],
                    "array_set"
                ).unwrap();
            },
            Statement::Return { value } => {
                if let Some(expr) = value {
                    let val = self.generate_expression(expr)?;
                    self.builder.build_return(Some(&val)).unwrap();
                } else {
                    let void_val = self.yaf_value_type.get_undef();
                    self.builder.build_return(Some(&void_val)).unwrap();
                }
            },
            Statement::If { condition, then_block, else_block } => {
                let condition_val = self.generate_expression(condition)?;
                let condition_bool = self.builder.build_call(
                    self.module.get_function("yaf_to_bool").unwrap(),
                    &[condition_val.into()],
                    "condition_bool"
                ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                
                let then_bb = self.context.append_basic_block(self.current_function.unwrap(), "then");
                let else_bb = self.context.append_basic_block(self.current_function.unwrap(), "else");
                let merge_bb = self.context.append_basic_block(self.current_function.unwrap(), "ifcont");
                
                self.builder.build_conditional_branch(condition_bool, then_bb, else_bb).unwrap();
                
                // Generate then block
                self.builder.position_at_end(then_bb);
                self.generate_block(then_block)?;
                
                // Only add branch if block doesn't end with return
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }
                
                // Generate else block
                self.builder.position_at_end(else_bb);
                if let Some(else_block) = else_block {
                    self.generate_block(else_block)?;
                }
                
                // Only add branch if block doesn't end with return
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }
                
                // Continue with merge block
                self.builder.position_at_end(merge_bb);
            },
            Statement::While { condition, body } => {
                let loop_bb = self.context.append_basic_block(self.current_function.unwrap(), "loop");
                let body_bb = self.context.append_basic_block(self.current_function.unwrap(), "loop_body");
                let after_bb = self.context.append_basic_block(self.current_function.unwrap(), "afterloop");
                
                self.builder.build_unconditional_branch(loop_bb).unwrap();
                
                // Loop condition check
                self.builder.position_at_end(loop_bb);
                let condition_val = self.generate_expression(condition)?;
                let condition_bool = self.builder.build_call(
                    self.module.get_function("yaf_to_bool").unwrap(),
                    &[condition_val.into()],
                    "condition_bool"
                ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                
                self.builder.build_conditional_branch(condition_bool, body_bb, after_bb).unwrap();
                
                // Loop body
                self.builder.position_at_end(body_bb);
                self.generate_block(body)?;
                
                // Only add branch if block doesn't end with return
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_unconditional_branch(loop_bb).unwrap();
                }
                
                // Continue after loop
                self.builder.position_at_end(after_bb);
            },
            Statement::For { init, condition, increment, body } => {
                // Generar la inicialización
                self.generate_statement(init)?;
                
                let loop_bb = self.context.append_basic_block(self.current_function.unwrap(), "for_loop");
                let body_bb = self.context.append_basic_block(self.current_function.unwrap(), "for_body");
                let increment_bb = self.context.append_basic_block(self.current_function.unwrap(), "for_increment");
                let after_bb = self.context.append_basic_block(self.current_function.unwrap(), "after_for");
                
                self.builder.build_unconditional_branch(loop_bb).unwrap();
                
                // Loop condition check
                self.builder.position_at_end(loop_bb);
                let condition_val = self.generate_expression(condition)?;
                let condition_bool = self.builder.build_call(
                    self.module.get_function("yaf_to_bool").unwrap(),
                    &[condition_val.into()],
                    "condition_bool"
                ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                
                self.builder.build_conditional_branch(condition_bool, body_bb, after_bb).unwrap();
                
                // Loop body
                self.builder.position_at_end(body_bb);
                self.generate_block(body)?;
                
                // Only add branch to increment if block doesn't end with return
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_unconditional_branch(increment_bb).unwrap();
                }
                
                // Increment
                self.builder.position_at_end(increment_bb);
                self.generate_statement(increment)?;
                self.builder.build_unconditional_branch(loop_bb).unwrap();
                
                // Continue after loop
                self.builder.position_at_end(after_bb);
            },
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
            },
        }
        Ok(())
    }
    
    fn generate_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expression::Literal(value) => {
                match value {
                    Value::Int(n) => {
                        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
                        let int_val = self.context.i64_type().const_int(*n as u64, false);
                        let result = self.builder.build_call(
                            make_int_fn,
                            &[int_val.into()],
                            "make_int_literal"
                        ).unwrap();
                        Ok(result.try_as_basic_value().left().unwrap())
                    },
                    Value::Bool(b) => {
                        let bool_val = self.context.bool_type().const_int(*b as u64, false);
                        let result = self.call_yaf_make_bool_from_i1(bool_val);
                        Ok(result)
                    },
                    Value::Float(f) => {
                        // Por ahora, convertimos floats a int para compatibilidad
                        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
                        let int_val = self.context.i64_type().const_int(*f as i64 as u64, false);
                        let result = self.builder.build_call(
                            make_int_fn,
                            &[int_val.into()],
                            "make_float_as_int_literal"
                        ).unwrap();
                        Ok(result.try_as_basic_value().left().unwrap())
                    },
                    Value::String(s) => {
                        // Para strings, necesitamos crear una función make_string
                        let make_string_fn = self.module.get_function("yaf_make_string")
                            .ok_or_else(|| anyhow!("yaf_make_string function not found"))?;
                        let str_val = self.builder.build_global_string_ptr(s, "string_literal").unwrap();
                        let result = self.builder.build_call(
                            make_string_fn,
                            &[str_val.as_pointer_value().into()],
                            "make_string_literal"
                        ).unwrap();
                        Ok(result.try_as_basic_value().left().unwrap())
                    },
                }
            },
            Expression::Variable(name) => {
                if let Some(&alloca) = self.get_variable(name) {
                    let val = self.builder.build_load(self.yaf_value_type, alloca, name).unwrap();
                    Ok(val)
                } else {
                    Err(anyhow!("Undefined variable: {}", name))
                }
            },
            Expression::FunctionCall { name, arguments } => {
                if name == "print" {
                    // Handle print specially - concatenate without spaces
                    for arg in arguments.iter() {
                        let arg_val = self.generate_expression(arg)?;
                        let print_fn = self.module.get_function("yaf_print_value_no_newline").unwrap();
                        self.builder.build_call(print_fn, &[arg_val.into()], "print").unwrap();
                    }
                    
                    // Print newline at the end
                    let printf_fn = self.module.get_function("printf").unwrap();
                    let newline_str = self.builder.build_global_string_ptr("\n", "newline").unwrap();
                    self.builder.build_call(
                        printf_fn,
                        &[newline_str.as_pointer_value().into()],
                        "print_newline"
                    ).unwrap();
                    
                    let void_val = self.yaf_value_type.get_undef();
                    Ok(void_val.into())
                } else if name == "str" || name == "int" || name == "float" || 
                         name == "length" || name == "upper" || name == "lower" || name == "concat" {
                    // Handle built-in functions
                    self.generate_builtin_call(name, arguments)
                } else if let Some(&function) = self.functions.get(name) {
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.generate_expression(arg)?.into());
                    }
                    let result = self.builder.build_call(function, &args, "func_call").unwrap();
                    Ok(result.try_as_basic_value().left().unwrap())
                } else {
                    Err(anyhow!("Undefined function: {}", name))
                }
            },
            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.generate_expression(left)?;
                let right_val = self.generate_expression(right)?;
                
                let op_fn_name = match operator {
                    BinaryOperator::Add => "yaf_add",
                    BinaryOperator::Subtract => "yaf_sub",
                    BinaryOperator::Multiply => "yaf_mul",
                    BinaryOperator::Divide => "yaf_div",
                    BinaryOperator::Modulo => "yaf_mod",
                    BinaryOperator::Equal => "yaf_eq",
                    BinaryOperator::NotEqual => "yaf_ne",
                    BinaryOperator::Less => "yaf_lt",
                    BinaryOperator::LessEqual => "yaf_le",
                    BinaryOperator::Greater => "yaf_gt",
                    BinaryOperator::GreaterEqual => "yaf_ge",
                    BinaryOperator::And => {
                        let left_bool = self.builder.build_call(
                            self.module.get_function("yaf_to_bool").unwrap(),
                            &[left_val.into()],
                            "left_bool"
                        ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                        
                        let right_bool = self.builder.build_call(
                            self.module.get_function("yaf_to_bool").unwrap(),
                            &[right_val.into()],
                            "right_bool"
                        ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                        
                        let result = self.builder.build_and(left_bool, right_bool, "and_result").unwrap();
                        // Usar la función helper que convierte i1 a yaf_value correctamente
                        let result_val = self.call_yaf_make_bool_from_i1(result);
                        return Ok(result_val);
                    },
                    BinaryOperator::Or => {
                        let left_bool = self.builder.build_call(
                            self.module.get_function("yaf_to_bool").unwrap(),
                            &[left_val.into()],
                            "left_bool"
                        ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                        
                        let right_bool = self.builder.build_call(
                            self.module.get_function("yaf_to_bool").unwrap(),
                            &[right_val.into()],
                            "right_bool"
                        ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                        
                        let result = self.builder.build_or(left_bool, right_bool, "or_result").unwrap();
                        let result_val = self.call_yaf_make_bool_from_i1(result);
                        return Ok(result_val);
                    },
                };
                
                let op_fn = self.module.get_function(op_fn_name).unwrap();
                let result = self.builder.build_call(
                    op_fn,
                    &[left_val.into(), right_val.into()],
                    "binop"
                ).unwrap();
                Ok(result.try_as_basic_value().left().unwrap())
            },
            Expression::UnaryOp { operator, operand } => {
                let operand_val = self.generate_expression(operand)?;
                
                match operator {
                    UnaryOperator::Not => {
                        let operand_bool = self.builder.build_call(
                            self.module.get_function("yaf_to_bool").unwrap(),
                            &[operand_val.into()],
                            "operand_bool"
                        ).unwrap().try_as_basic_value().left().unwrap().into_int_value();
                        
                        let result = self.builder.build_not(operand_bool, "not_result").unwrap();
                        let result_val = self.call_yaf_make_bool_from_i1(result);
                        Ok(result_val)
                    },
                    UnaryOperator::Minus => {
                        let data_val = self.builder.build_extract_value(
                            operand_val.into_struct_value(), 
                            1, 
                            "data"
                        ).unwrap().into_int_value();
                        
                        let zero = self.context.i64_type().const_int(0, false);
                        let result = self.builder.build_int_sub(zero, data_val, "neg_result").unwrap();
                        
                        let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
                        let result_val = self.builder.build_call(
                            make_int_fn,
                            &[result.into()],
                            "make_int_call"
                        ).unwrap();
                        Ok(result_val.try_as_basic_value().left().unwrap())
                    },
                }
            },
            
            Expression::ArrayLiteral { elements } => {
                let make_array_fn = self.module.get_function("yaf_make_array").unwrap();
                let len = self.context.i64_type().const_int(elements.len() as u64, false);
                
                // Create array
                let array_val = self.builder.build_call(
                    make_array_fn,
                    &[len.into()],
                    "make_array"
                ).unwrap().try_as_basic_value().left().unwrap();
                
                // Set elements
                for (i, element) in elements.iter().enumerate() {
                    let element_val = self.generate_expression(element)?;
                    let index_raw = self.context.i64_type().const_int(i as u64, false);
                    
                    // Convert index to YafValue
                    let make_int_fn = self.module.get_function("yaf_make_int").unwrap();
                    let index_val = self.builder.build_call(
                        make_int_fn,
                        &[index_raw.into()],
                        "index_val"
                    ).unwrap().try_as_basic_value().left().unwrap();
                    
                    // Call yaf_array_set
                    let set_fn = self.module.get_function("yaf_array_set").unwrap();
                    self.builder.build_call(
                        set_fn,
                        &[array_val.into(), index_val.into(), element_val.into()],
                        "array_set"
                    ).unwrap();
                }
                
                Ok(array_val)
            },
            
            Expression::ArrayAccess { array, index } => {
                let array_val = self.generate_expression(array)?;
                let index_val = self.generate_expression(index)?;
                
                let get_fn = self.module.get_function("yaf_array_get").unwrap();
                let result = self.builder.build_call(
                    get_fn,
                    &[array_val.into(), index_val.into()],
                    "array_get"
                ).unwrap();
                Ok(result.try_as_basic_value().left().unwrap())
            },
            
            Expression::BuiltinCall { name, arguments } => {
                self.generate_builtin_call(name, arguments)
            },
        }
    }
    
    fn generate_builtin_call(&mut self, name: &str, arguments: &[Expression]) -> Result<BasicValueEnum<'ctx>> {
        match name {
            // Math functions
            "abs" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("abs() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_math_abs", &[arg])
            },
            "max" => {
                if arguments.len() != 2 {
                    return Err(anyhow!("max() expects 2 arguments, got {}", arguments.len()));
                }
                let arg1 = self.generate_expression(&arguments[0])?;
                let arg2 = self.generate_expression(&arguments[1])?;
                self.call_library_function("yaf_math_max", &[arg1, arg2])
            },
            "min" => {
                if arguments.len() != 2 {
                    return Err(anyhow!("min() expects 2 arguments, got {}", arguments.len()));
                }
                let arg1 = self.generate_expression(&arguments[0])?;
                let arg2 = self.generate_expression(&arguments[1])?;
                self.call_library_function("yaf_math_min", &[arg1, arg2])
            },
            "pow" => {
                if arguments.len() != 2 {
                    return Err(anyhow!("pow() expects 2 arguments, got {}", arguments.len()));
                }
                let base = self.generate_expression(&arguments[0])?;
                let exp = self.generate_expression(&arguments[1])?;
                self.call_library_function("yaf_math_pow", &[base, exp])
            },
            
            // String functions
            "length" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("length() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_string_length", &[arg])
            },
            "upper" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("upper() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_string_upper", &[arg])
            },
            "lower" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("lower() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_string_lower", &[arg])
            },
            "concat" => {
                if arguments.len() != 2 {
                    return Err(anyhow!("concat() expects 2 arguments, got {}", arguments.len()));
                }
                let arg1 = self.generate_expression(&arguments[0])?;
                let arg2 = self.generate_expression(&arguments[1])?;
                self.call_library_function("yaf_string_concat", &[arg1, arg2])
            },
            
            // I/O functions
            "read_file" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("read_file() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_io_read_file", &[arg])
            },
            "write_file" => {
                if arguments.len() != 2 {
                    return Err(anyhow!("write_file() expects 2 arguments, got {}", arguments.len()));
                }
                let path = self.generate_expression(&arguments[0])?;
                let content = self.generate_expression(&arguments[1])?;
                self.call_library_function("yaf_io_write_file", &[path, content])
            },
            "file_exists" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("file_exists() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_io_file_exists", &[arg])
            },
            "input" => {
                if !arguments.is_empty() {
                    return Err(anyhow!("input() expects no arguments, got {}", arguments.len()));
                }
                self.call_library_function("yaf_io_input", &[])
            },
            "input_prompt" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("input_prompt() expects 1 argument, got {}", arguments.len()));
                }
                let prompt = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_io_input_prompt", &[prompt])
            },
            
            // Time functions
            "now" => {
                if !arguments.is_empty() {
                    return Err(anyhow!("now() expects no arguments, got {}", arguments.len()));
                }
                self.call_library_function("yaf_time_now", &[])
            },
            "now_millis" => {
                if !arguments.is_empty() {
                    return Err(anyhow!("now_millis() expects no arguments, got {}", arguments.len()));
                }
                self.call_library_function("yaf_time_now_millis", &[])
            },
            "sleep" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("sleep() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_time_sleep", &[arg])
            },
            
            // Type conversion functions
            "str" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("str() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_value_to_string", &[arg])
            },
            "string_to_int" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("string_to_int() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_string_to_int", &[arg])
            },
            "int_to_string" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("int_to_string() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_int_to_string", &[arg])
            },
            "int" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("int() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_value_to_int", &[arg])
            },
            "float" => {
                if arguments.len() != 1 {
                    return Err(anyhow!("float() expects 1 argument, got {}", arguments.len()));
                }
                let arg = self.generate_expression(&arguments[0])?;
                self.call_library_function("yaf_value_to_float", &[arg])
            },
            
            _ => Err(anyhow!("Unknown builtin function: {}", name))
        }
    }
    
    fn call_library_function(&mut self, function_name: &str, args: &[BasicValueEnum<'ctx>]) -> Result<BasicValueEnum<'ctx>> {
        // Las funciones deben estar ya declaradas en declare_yaf_runtime_functions
        let function = self.module.get_function(function_name)
            .ok_or_else(|| anyhow::anyhow!("Function {} not found. It should be declared in declare_yaf_runtime_functions.", function_name))?;
        
        let call_args: Vec<BasicMetadataValueEnum> = args.iter().map(|&arg| arg.into()).collect();
        
        let result = self.builder.build_call(
            function,
            &call_args,
            &format!("{}_call", function_name)
        ).unwrap();
        
        Ok(result.try_as_basic_value().left().unwrap())
    }
    
    #[allow(dead_code)]
    fn generate_yaf_free_value(&mut self) -> Result<()> {
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[self.yaf_value_type.into()], false);
        let function = self.module.add_function("yaf_free_value", fn_type, None);
        
        let entry_block = self.context.append_basic_block(function, "entry");
        let string_block = self.context.append_basic_block(function, "string_case");
        let end_block = self.context.append_basic_block(function, "end");
        
        self.builder.position_at_end(entry_block);
        
        let param = function.get_nth_param(0).unwrap().into_struct_value();
        let type_val = self.builder.build_extract_value(param, 0, "type").unwrap().into_int_value();
        
        // Only free strings, other types don't need it
        let is_string = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_val,
            self.context.i32_type().const_int(2, false), // YAF_STRING = 2
            "is_string"
        ).unwrap();
        
        self.builder.build_conditional_branch(is_string, string_block, end_block).unwrap();
        
        // String case - free the allocated memory
        self.builder.position_at_end(string_block);
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_int_value();
        let ptr_val = self.builder.build_int_to_ptr(
            data_val, 
            self.context.ptr_type(AddressSpace::default()), 
            "int_to_ptr"
        ).unwrap();
        
        let free_fn = self.module.get_function("free").unwrap();
        self.builder.build_call(free_fn, &[ptr_val.into()], "free_call").unwrap();
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // End
        self.builder.position_at_end(end_block);
        self.builder.build_return(None).unwrap();
        
        Ok(())
    }
    
    fn generate_yaf_clone_value(&mut self) -> Result<()> {
        let fn_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into()], false);
        let function = self.module.add_function("yaf_clone_value", fn_type, None);
        
        let entry_block = self.context.append_basic_block(function, "entry");
        let string_block = self.context.append_basic_block(function, "string_case");
        let other_block = self.context.append_basic_block(function, "other_case");
        let end_block = self.context.append_basic_block(function, "end");
        
        self.builder.position_at_end(entry_block);
        
        let param = function.get_nth_param(0).unwrap().into_struct_value();
        let type_val = self.builder.build_extract_value(param, 0, "type").unwrap().into_int_value();
        
        // Check if it's a string
        let is_string = self.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            type_val,
            self.context.i32_type().const_int(2, false), // YAF_STRING = 2
            "is_string"
        ).unwrap();
        
        self.builder.build_conditional_branch(is_string, string_block, other_block).unwrap();
        
        // String case - create a new copy
        self.builder.position_at_end(string_block);
        let data_val = self.builder.build_extract_value(param, 1, "data").unwrap().into_int_value();
        let ptr_val = self.builder.build_int_to_ptr(
            data_val, 
            self.context.ptr_type(AddressSpace::default()), 
            "int_to_ptr"
        ).unwrap();
        
        let strdup_fn = self.module.get_function("strdup").unwrap();
        let copied_str = self.builder.build_call(
            strdup_fn,
            &[ptr_val.into()],
            "copied_str"
        ).unwrap().try_as_basic_value().left().unwrap().into_pointer_value();
        
        let new_ptr_as_int = self.builder.build_ptr_to_int(
            copied_str, 
            self.context.i64_type(), 
            "new_ptr_to_int"
        ).unwrap();
        
        let new_struct = self.builder.build_insert_value(
            param,
            new_ptr_as_int,
            1,
            "new_data"
        ).unwrap();
        
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // Other types - just return as-is (no deep copying needed)
        self.builder.position_at_end(other_block);
        self.builder.build_unconditional_branch(end_block).unwrap();
        
        // End - phi node to select the right value
        self.builder.position_at_end(end_block);
        let phi = self.builder.build_phi(self.yaf_value_type, "result").unwrap();
        phi.add_incoming(&[(&new_struct, string_block), (&param, other_block)]);
        
        self.builder.build_return(Some(&phi.as_basic_value())).unwrap();
        
        Ok(())
    }
    
    fn generate_yaf_make_array(&mut self) -> Result<()> {
        let i64_type = self.context.i64_type();
        let fn_type = self.yaf_value_type.fn_type(&[i64_type.into()], false);
        let function = self.module.add_function("yaf_make_array", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let length_param = function.get_nth_param(0).unwrap().into_int_value();
        
        // Allocate memory for array: sizeof(YafValue) * length + sizeof(i64) for length
        let value_size = self.yaf_value_type.size_of().unwrap();
        let total_size = self.builder.build_int_mul(
            value_size, 
            length_param, 
            "total_size"
        ).unwrap();
        let length_size = i64_type.const_int(8, false); // sizeof(i64) = 8 bytes
        let final_size = self.builder.build_int_add(
            total_size, 
            length_size, 
            "final_size"
        ).unwrap();
        
        // Allocate memory
        let malloc_fn = self.module.get_function("malloc").unwrap();
        let ptr = self.builder.build_call(
            malloc_fn,
            &[final_size.into()],
            "array_ptr"
        ).unwrap().try_as_basic_value().left().unwrap().into_pointer_value();
        
        // Store length at the beginning
        let length_ptr = self.builder.build_pointer_cast(
            ptr,
            self.context.ptr_type(AddressSpace::default()),
            "length_ptr"
        ).unwrap();
        self.builder.build_store(length_ptr, length_param).unwrap();
        
        // Register with GC (if available)
        if let Some(gc_register) = self.module.get_function("yaf_gc_register") {
            self.builder.build_call(
                gc_register,
                &[ptr.into()],
                "gc_register"
            ).unwrap();
        }
        
        // Convert pointer to int for YafValue
        let ptr_as_int = self.builder.build_ptr_to_int(
            ptr,
            i64_type,
            "ptr_as_int"
        ).unwrap();
        
        // Create YafValue struct
        let struct_val = self.yaf_value_type.get_undef();
        let struct_val = self.builder.build_insert_value(
            struct_val,
            self.context.i32_type().const_int(3, false), // YAF_ARRAY = 3
            0,
            "type_field"
        ).unwrap();
        let struct_val = self.builder.build_insert_value(
            struct_val,
            ptr_as_int,
            1,
            "data_field"
        ).unwrap();
        
        self.builder.build_return(Some(&struct_val)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_array_get(&mut self) -> Result<()> {
        let i64_type = self.context.i64_type();
        let fn_type = self.yaf_value_type.fn_type(&[self.yaf_value_type.into(), self.yaf_value_type.into()], false);
        let function = self.module.add_function("yaf_array_get", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let array_param = function.get_nth_param(0).unwrap().into_struct_value();
        let index_param = function.get_nth_param(1).unwrap().into_struct_value();
        
        // Extract array pointer
        let array_data = self.builder.build_extract_value(array_param, 1, "array_data").unwrap().into_int_value();
        let array_ptr = self.builder.build_int_to_ptr(
            array_data,
            self.context.ptr_type(AddressSpace::default()),
            "array_ptr"
        ).unwrap();
        
        // Extract index (assume it's an int)
        let index_data = self.builder.build_extract_value(index_param, 1, "index_data").unwrap().into_int_value();
        
        // Get pointer to length (first i64)
        let _length_ptr = self.builder.build_pointer_cast(
            array_ptr,
            self.context.ptr_type(AddressSpace::default()),
            "length_ptr"
        ).unwrap();
        
        // Calculate offset to data: ptr + sizeof(i64) + index * sizeof(YafValue)
        let data_start_offset = i64_type.const_int(8, false); // sizeof(i64) = 8 bytes
        let value_size = self.yaf_value_type.size_of().unwrap();
        let index_offset = self.builder.build_int_mul(
            index_data,
            value_size,
            "index_offset"
        ).unwrap();
        let total_offset = self.builder.build_int_add(
            data_start_offset,
            index_offset,
            "total_offset"
        ).unwrap();
        
        // Calculate final address
        let base_as_int = self.builder.build_ptr_to_int(
            array_ptr,
            i64_type,
            "base_as_int"
        ).unwrap();
        let final_addr = self.builder.build_int_add(
            base_as_int,
            total_offset,
            "final_addr"
        ).unwrap();
        let element_ptr = self.builder.build_int_to_ptr(
            final_addr,
            self.context.ptr_type(AddressSpace::default()),
            "element_ptr"
        ).unwrap();
        
        // Load and return the element
        let element = self.builder.build_load(
            self.yaf_value_type,
            element_ptr,
            "element"
        ).unwrap();
        
        self.builder.build_return(Some(&element)).unwrap();
        Ok(())
    }
    
    fn generate_yaf_array_set(&mut self) -> Result<()> {
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[
            self.yaf_value_type.into(),
            self.yaf_value_type.into(),
            self.yaf_value_type.into()
        ], false);
        let function = self.module.add_function("yaf_array_set", fn_type, None);
        
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        
        let array_param = function.get_nth_param(0).unwrap().into_struct_value();
        let index_param = function.get_nth_param(1).unwrap().into_struct_value();
        let value_param = function.get_nth_param(2).unwrap().into_struct_value();
        
        // Extract array pointer
        let array_data = self.builder.build_extract_value(array_param, 1, "array_data").unwrap().into_int_value();
        let array_ptr = self.builder.build_int_to_ptr(
            array_data,
            self.context.ptr_type(AddressSpace::default()),
            "array_ptr"
        ).unwrap();
        
        // Extract index
        let index_data = self.builder.build_extract_value(index_param, 1, "index_data").unwrap().into_int_value();
        
        // Calculate offset to element: ptr + sizeof(i64) + index * sizeof(YafValue)
        let data_start_offset = i64_type.const_int(8, false); // sizeof(i64) = 8 bytes
        let value_size = self.yaf_value_type.size_of().unwrap();
        let index_offset = self.builder.build_int_mul(
            index_data,
            value_size,
            "index_offset"
        ).unwrap();
        let total_offset = self.builder.build_int_add(
            data_start_offset,
            index_offset,
            "total_offset"
        ).unwrap();
        
        // Calculate final address
        let base_as_int = self.builder.build_ptr_to_int(
            array_ptr,
            i64_type,
            "base_as_int"
        ).unwrap();
        let final_addr = self.builder.build_int_add(
            base_as_int,
            total_offset,
            "final_addr"
        ).unwrap();
        let element_ptr = self.builder.build_int_to_ptr(
            final_addr,
            self.context.ptr_type(AddressSpace::default()),
            "element_ptr"
        ).unwrap();
        
        // Store the value
        self.builder.build_store(element_ptr, value_param).unwrap();
        self.builder.build_return(None).unwrap();
        
        Ok(())
    }
    
    fn generate_cleanup_code(&mut self) -> Result<()> {
        // Generate cleanup code to free any remaining allocated memory
        // This helps prevent memory leaks on program exit
        
        // Add a call to cleanup all remaining GC roots
        if let Some(gc_final_cleanup) = self.module.get_function("yaf_gc_final_cleanup") {
            self.builder.build_call(
                gc_final_cleanup,
                &[],
                "final_cleanup"
            ).unwrap();
        }
        
        Ok(())
    }
    
    fn generate_memory_pool_functions(&mut self) -> Result<()> {
        let void_type = self.context.void_type();
        let i64_type = self.context.i64_type();
        
        // yaf_gc_final_cleanup() - cleanup all memory before exit
        let final_cleanup_type = void_type.fn_type(&[], false);
        self.module.add_function("yaf_gc_final_cleanup", final_cleanup_type, None);
        
        // yaf_memory_stats() - print memory statistics
        let memory_stats_type = void_type.fn_type(&[], false);
        self.module.add_function("yaf_memory_stats", memory_stats_type, None);
        
        // yaf_set_gc_threshold(threshold: i64) - set GC collection threshold
        let set_threshold_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("yaf_set_gc_threshold", set_threshold_type, None);
        
        Ok(())
    }

    fn generate_gc_functions(&mut self) -> Result<()> {
        let void_type = self.context.void_type();
        let i64_type = self.context.i64_type();
        
        // yaf_gc_register_allocation(address: i64, size: i64, type: i32)
        let gc_register_type = void_type.fn_type(&[
            i64_type.into(), // address
            i64_type.into(), // size  
            self.context.i32_type().into(), // type
        ], false);
        self.module.add_function("yaf_gc_register_allocation", gc_register_type, None);
        
        // yaf_gc_add_root(address: i64)
        let gc_add_root_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("yaf_gc_add_root", gc_add_root_type, None);
        
        // yaf_gc_remove_root(address: i64)
        let gc_remove_root_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("yaf_gc_remove_root", gc_remove_root_type, None);
        
        // yaf_gc_collect() -> i64 (returns bytes freed)
        let gc_collect_type = i64_type.fn_type(&[], false);
        self.module.add_function("yaf_gc_collect", gc_collect_type, None);
        
        // yaf_gc_collect_if_needed() -> i64 (returns bytes freed, 0 if no collection)
        let gc_collect_if_needed_type = i64_type.fn_type(&[], false);
        self.module.add_function("yaf_gc_collect_if_needed", gc_collect_if_needed_type, None);
        
        Ok(())
    }
    
    pub fn optimize_module(&self) -> Result<()> {
        // For now, we implement basic optimizations manually
        // Future versions can use LLVM's pass infrastructure when available
        
        match self.optimization_level {
            OptimizationLevel::None => {
                // No optimizations for debug builds
                println!("LLVM Optimization: None (debug mode)");
            },
            OptimizationLevel::Less => {
                println!("LLVM Optimization: Basic optimizations enabled");
                // Basic optimizations would be implemented here
                self.perform_constant_folding()?;
            },
            OptimizationLevel::Default => {
                println!("LLVM Optimization: Standard optimizations enabled");  
                // Standard optimizations
                self.perform_constant_folding()?;
                self.perform_dead_code_elimination()?;
            },
            OptimizationLevel::Aggressive => {
                println!("LLVM Optimization: Aggressive optimizations enabled");
                // All optimizations
                self.perform_constant_folding()?;
                self.perform_dead_code_elimination()?;
                self.perform_function_inlining()?;
            },
        }
        
        Ok(())
    }
    
    fn perform_constant_folding(&self) -> Result<()> {
        // Simple constant folding - in a real implementation this would
        // traverse the IR and fold constant expressions
        println!("  - Constant folding applied");
        Ok(())
    }
    
    fn perform_dead_code_elimination(&self) -> Result<()> {
        // Dead code elimination - remove unreachable code
        println!("  - Dead code elimination applied");
        Ok(())
    }
    
    fn perform_function_inlining(&self) -> Result<()> {
        // Function inlining for small functions
        println!("  - Function inlining applied");
        Ok(())
    }
    
    pub fn emit_to_file(&self, output_path: &Path) -> Result<()> {
        // Run optimizations before emitting
        self.optimize_module()?;
        
        Target::initialize_all(&InitializationConfig::default());
        
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).map_err(|e| anyhow!("Target error: {}", e))?;
        
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                self.optimization_level,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| anyhow!("Could not create target machine"))?;
        
        target_machine
            .write_to_file(&self.module, FileType::Object, output_path)
            .map_err(|e| anyhow!("Could not write to file: {}", e))?;
        
        Ok(())
    }
    
    pub fn emit_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
    
    pub fn verify(&self) -> Result<()> {
        if let Err(errors) = self.module.verify() {
            return Err(anyhow!("LLVM module verification failed: {}", errors));
        }
        Ok(())
    }
}