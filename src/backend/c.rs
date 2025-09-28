use crate::core::ast::*;
use crate::runtime::values::Value;
use crate::error::{Result, YafError};
use std::collections::{HashMap, HashSet};

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    functions: HashMap<String, Function>,
    in_function: bool,
    declared_vars: HashSet<String>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            indent_level: 0,
            functions: HashMap::new(),
            in_function: false,
            declared_vars: HashSet::new(),
        }
    }
    
    pub fn generate(&mut self, program: Program) -> Result<String> {
        // Registrar funciones
        for function in &program.functions {
            self.functions.insert(function.name.clone(), function.clone());
        }
        
        // Generar includes y usar el runtime YAF existente  
        self.emit_line("#include <stdio.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("#include <ctype.h>");
        self.emit_line("#include <string.h>");
        
        // Incluir declaraciones del runtime directamente
        self.emit_line("// YAF Runtime declarations");
        self.emit_line("typedef enum {");
        self.emit_line("    YAF_INT,");
        self.emit_line("    YAF_BOOL,");
        self.emit_line("    YAF_STRING,");
        self.emit_line("    YAF_VOID");
        self.emit_line("} yaf_type_t;");
        self.emit_line("");
        self.emit_line("typedef struct {");
        self.emit_line("    yaf_type_t type;");
        self.emit_line("    union {");
        self.emit_line("        int int_val;");
        self.emit_line("        bool bool_val;");
        self.emit_line("        char* string_val;");
        self.emit_line("    } data;");
        self.emit_line("} yaf_value_t;");
        self.emit_line("");
        
        self.emit_line("#define YAF_INT    0");
        self.emit_line("#define YAF_BOOL   1");
        self.emit_line("#define YAF_STRING 2");
        self.emit_line("#define YAF_VOID   3");
        self.emit_line("");
        
        self.emit_line("void yaf_print_value(yaf_value_t value);");
        self.emit_line("yaf_value_t yaf_make_int(int val);");
        self.emit_line("yaf_value_t yaf_make_bool(bool val);");
        self.emit_line("yaf_value_t yaf_make_string(char* val);");
        self.emit_line("yaf_value_t yaf_make_void();");
        self.emit_line("yaf_value_t yaf_make_array(void);");
        self.emit_line("void yaf_print_value(yaf_value_t value);");
        self.emit_line("bool yaf_to_bool(yaf_value_t val);");
        self.emit_line("yaf_value_t yaf_eq(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_lt(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_gt(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_le(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_ge(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_add(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_sub(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_mul(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_div(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_mod(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_math_abs(yaf_value_t value);");
        self.emit_line("yaf_value_t yaf_math_max(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_math_min(yaf_value_t a, yaf_value_t b);");
        self.emit_line("yaf_value_t yaf_math_pow(yaf_value_t base, yaf_value_t exp);");
        self.emit_line("yaf_value_t yaf_string_length(yaf_value_t str);");
        self.emit_line("yaf_value_t yaf_string_upper(yaf_value_t str);");
        self.emit_line("yaf_value_t yaf_string_lower(yaf_value_t str);");
        self.emit_line("yaf_value_t yaf_string_concat(yaf_value_t a, yaf_value_t b);");
        self.emit_line("");
        self.emit_line("bool yaf_to_bool(yaf_value_t val) {");
        self.emit_line("    switch (val.type) {");
        self.emit_line("        case YAF_BOOL: return val.data.bool_val;");
        self.emit_line("        case YAF_INT: return val.data.int_val != 0;");
        self.emit_line("        default: return false;");
        self.emit_line("    }");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_eq(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type != b.type) return yaf_make_bool(0);");
        self.emit_line("    switch (a.type) {");
        self.emit_line("        case YAF_INT: return yaf_make_bool(a.data.int_val == b.data.int_val);");
        self.emit_line("        case YAF_BOOL: return yaf_make_bool(a.data.bool_val == b.data.bool_val);");
        self.emit_line("        default: return yaf_make_bool(0);");
        self.emit_line("    }");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_lt(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_bool(a.data.int_val < b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_bool(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_gt(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_bool(a.data.int_val > b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_bool(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_le(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_bool(a.data.int_val <= b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_bool(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_ge(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_bool(a.data.int_val >= b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_bool(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_add(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_int(a.data.int_val + b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    if (a.type == YAF_STRING || b.type == YAF_STRING) {");
        self.emit_line("        // String concatenation - simplified");
        self.emit_line("        return a; // Return first for now");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_int(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_sub(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_int(a.data.int_val - b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_int(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_mul(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.emit_line("        return yaf_make_int(a.data.int_val * b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_int(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_div(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT && b.data.int_val != 0) {");
        self.emit_line("        return yaf_make_int(a.data.int_val / b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_int(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_mod(yaf_value_t a, yaf_value_t b) {");
        self.emit_line("    if (a.type == YAF_INT && b.type == YAF_INT && b.data.int_val != 0) {");
        self.emit_line("        return yaf_make_int(a.data.int_val % b.data.int_val);");
        self.emit_line("    }");
        self.emit_line("    return yaf_make_int(0);");
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_make_array(void) {");
        self.emit_line("    yaf_value_t val;");
        self.emit_line("    val.type = YAF_INT; // Simplified - arrays return int 0");
        self.emit_line("    val.data.int_val = 0;");
        self.emit_line("    return val;");
        self.emit_line("}");
        self.emit_line("");
        
        // Las funciones auxiliares están en el runtime YAF
        
        // Declaraciones de funciones de usuario
        for function in &program.functions {
            self.generate_function_declaration(function)?;
        }
        self.emit_line("");
        
        // Generar funciones helper
        self.generate_helper_functions();
        
        // Implementaciones de funciones de usuario
        for function in &program.functions {
            self.generate_function(function)?;
        }
        
        // Función main
        self.emit_line("int main() {");
        self.indent();
        self.generate_block(&program.main)?;
        self.emit_line("return 0;");
        self.dedent();
        self.emit_line("}");
        
        Ok(self.output.clone())
    }
    
    #[allow(dead_code)]
    fn generate_helper_functions(&mut self) {
        // Función para imprimir
        self.emit_line("void yaf_print_value(yaf_value_t val) {");
        self.indent();
        self.emit_line("switch (val.type) {");
        self.indent();
        self.emit_line("case YAF_INT:");
        self.indent();
        self.emit_line("printf(\"%d\", val.data.int_val);");
        self.emit_line("break;");
        self.dedent();
        self.emit_line("case YAF_STRING:");
        self.indent();
        self.emit_line("printf(\"%s\", val.data.string_val);");
        self.emit_line("break;");
        self.dedent();
        self.emit_line("case YAF_BOOL:");
        self.indent();
        self.emit_line("printf(\"%s\", val.data.bool_val ? \"true\" : \"false\");");
        self.emit_line("break;");
        self.dedent();
        self.emit_line("case YAF_VOID:");
        self.indent();
        self.emit_line("printf(\"void\");");
        self.emit_line("break;");
        self.dedent();
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        // Función para crear valores
        self.emit_line("yaf_value_t yaf_make_int(int val) {");
        self.indent();
        self.emit_line("yaf_value_t result;");
        self.emit_line("result.type = YAF_INT;");
        self.emit_line("result.data.int_val = val;");
        self.emit_line("return result;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_make_bool(bool val) {");
        self.indent();
        self.emit_line("yaf_value_t result;");
        self.emit_line("result.type = YAF_BOOL;");
        self.emit_line("result.data.bool_val = val;");
        self.emit_line("return result;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_make_string(const char* val) {");
        self.indent();
        self.emit_line("yaf_value_t result;");
        self.emit_line("result.type = YAF_STRING;");
        self.emit_line("result.data.string_val = strdup(val);");
        self.emit_line("return result;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_make_void() {");
        self.indent();
        self.emit_line("yaf_value_t result;");
        self.emit_line("result.type = YAF_VOID;");
        self.emit_line("return result;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        // Operaciones aritméticas
        self.emit_line("yaf_value_t yaf_add(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_int(a.data.int_val + b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Operación de suma inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_sub(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_int(a.data.int_val - b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Operación de resta inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_mul(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_int(a.data.int_val * b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Operación de multiplicación inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_div(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("if (b.data.int_val == 0) {");
        self.indent();
        self.emit_line("fprintf(stderr, \"Error: División por cero\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(a.data.int_val / b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Operación de división inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_mod(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("if (b.data.int_val == 0) {");
        self.indent();
        self.emit_line("fprintf(stderr, \"Error: Módulo por cero\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(a.data.int_val % b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Operación de módulo inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        // Operaciones de comparación
        self.emit_line("yaf_value_t yaf_eq(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type != b.type) return yaf_make_bool(false);");
        self.emit_line("switch (a.type) {");
        self.indent();
        self.emit_line("case YAF_INT: return yaf_make_bool(a.data.int_val == b.data.int_val);");
        self.emit_line("case YAF_BOOL: return yaf_make_bool(a.data.bool_val == b.data.bool_val);");
        self.emit_line("case YAF_STRING: return yaf_make_bool(strcmp(a.data.string_val, b.data.string_val) == 0);");
        self.emit_line("default: return yaf_make_bool(false);");
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_lt(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_bool(a.data.int_val < b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Comparación inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_le(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_bool(a.data.int_val <= b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Comparación inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_gt(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_bool(a.data.int_val > b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Comparación inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_ge(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return yaf_make_bool(a.data.int_val >= b.data.int_val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("fprintf(stderr, \"Error: Comparación inválida\\n\");");
        self.emit_line("exit(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        // Función para convertir a bool
        self.emit_line("bool yaf_to_bool(yaf_value_t val) {");
        self.indent();
        self.emit_line("switch (val.type) {");
        self.indent();
        self.emit_line("case YAF_BOOL: return val.data.bool_val;");
        self.emit_line("case YAF_INT: return val.data.int_val != 0;");
        self.emit_line("case YAF_STRING: return strlen(val.data.string_val) > 0;");
        self.emit_line("default: return false;");
        self.dedent();
        self.emit_line("}");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        // Funciones matemáticas
        self.emit_line("yaf_value_t yaf_math_abs(yaf_value_t value) {");
        self.indent();
        self.emit_line("if (value.type == YAF_INT) {");
        self.indent();
        self.emit_line("int val = value.data.int_val;");
        self.emit_line("return yaf_make_int(val < 0 ? -val : val);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(0);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_math_max(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return a.data.int_val > b.data.int_val ? a : b;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(0);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_math_min(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_INT && b.type == YAF_INT) {");
        self.indent();
        self.emit_line("return a.data.int_val < b.data.int_val ? a : b;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(0);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_math_pow(yaf_value_t base, yaf_value_t exp) {");
        self.indent();
        self.emit_line("if (base.type == YAF_INT && exp.type == YAF_INT) {");
        self.indent();
        self.emit_line("int result = 1;");
        self.emit_line("int b = base.data.int_val;");
        self.emit_line("int e = exp.data.int_val;");
        self.emit_line("for (int i = 0; i < e; i++) {");
        self.indent();
        self.emit_line("result *= b;");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(result);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(1);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        // Funciones de strings
        self.emit_line("yaf_value_t yaf_string_length(yaf_value_t str) {");
        self.indent();
        self.emit_line("if (str.type == YAF_STRING) {");
        self.indent();
        self.emit_line("return yaf_make_int(strlen(str.data.string_val));");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_int(0);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_string_upper(yaf_value_t str) {");
        self.indent();
        self.emit_line("if (str.type == YAF_STRING) {");
        self.indent();
        self.emit_line("int len = strlen(str.data.string_val);");
        self.emit_line("char* result = malloc(len + 1);");
        self.emit_line("for (int i = 0; i < len; i++) {");
        self.indent();
        self.emit_line("result[i] = toupper(str.data.string_val[i]);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("result[len] = '\\0';");
        self.emit_line("return yaf_make_string(result);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_string(\"\");");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_string_lower(yaf_value_t str) {");
        self.indent();
        self.emit_line("if (str.type == YAF_STRING) {");
        self.indent();
        self.emit_line("int len = strlen(str.data.string_val);");
        self.emit_line("char* result = malloc(len + 1);");
        self.emit_line("for (int i = 0; i < len; i++) {");
        self.indent();
        self.emit_line("result[i] = tolower(str.data.string_val[i]);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("result[len] = '\\0';");
        self.emit_line("return yaf_make_string(result);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_string(\"\");");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.emit_line("yaf_value_t yaf_string_concat(yaf_value_t a, yaf_value_t b) {");
        self.indent();
        self.emit_line("if (a.type == YAF_STRING && b.type == YAF_STRING) {");
        self.indent();
        self.emit_line("int len_a = strlen(a.data.string_val);");
        self.emit_line("int len_b = strlen(b.data.string_val);");
        self.emit_line("char* result = malloc(len_a + len_b + 1);");
        self.emit_line("strcpy(result, a.data.string_val);");
        self.emit_line("strcat(result, b.data.string_val);");
        self.emit_line("return yaf_make_string(result);");
        self.dedent();
        self.emit_line("}");
        self.emit_line("return yaf_make_string(\"\");");
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        

    }
    
    fn generate_function_declaration(&mut self, function: &Function) -> Result<()> {
        let return_type = "yaf_value_t"; // Siempre usar yaf_value_t para consistencia
        
        let mut decl = format!("{} yaf_func_{}(", return_type, function.name);
        
        for (i, param) in function.parameters.iter().enumerate() {
            if i > 0 {
                decl.push_str(", ");
            }
            decl.push_str("yaf_value_t ");
            decl.push_str(&param.name);
        }
        
        decl.push_str(");");
        self.emit_line(&decl);
        
        Ok(())
    }
    
    fn generate_function(&mut self, function: &Function) -> Result<()> {
        self.in_function = true;
        self.declared_vars.clear(); // Nueva función, limpiar variables
        
        let return_type = "yaf_value_t"; // Siempre usar yaf_value_t para consistencia
        
        let mut def = format!("{} yaf_func_{}(", return_type, function.name);
        
        for (i, param) in function.parameters.iter().enumerate() {
            if i > 0 {
                def.push_str(", ");
            }
            def.push_str("yaf_value_t ");
            def.push_str(&param.name);
        }
        
        def.push_str(") {");
        self.emit_line(&def);
        self.indent();
        
        self.generate_block(&function.body)?;
        
        // Si no hay return explícito, agregar return void
        self.emit_line("return yaf_make_void();");
        
        self.dedent();
        self.emit_line("}");
        self.emit_line("");
        
        self.in_function = false;
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
                let expr_result = self.generate_expression(value)?;
                // Para declaración, siempre crear nueva variable
                self.emit_line(&format!("yaf_value_t {} = {};", name, expr_result));
                self.declared_vars.insert(name.clone());
            },
            Statement::Assignment { name, value } => {
                let expr_result = self.generate_expression(value)?;
                if self.declared_vars.contains(name) {
                    // Variable ya declarada, hacer asignación
                    self.emit_line(&format!("{} = {};", name, expr_result));
                } else {
                    // Variable nueva, hacer declaración
                    self.emit_line(&format!("yaf_value_t {} = {};", name, expr_result));
                    self.declared_vars.insert(name.clone());
                }
            },
            
            Statement::ArrayAssignment { name, index, value } => {
                let index_result = self.generate_expression(index)?;
                let value_result = self.generate_expression(value)?;
                self.emit_line(&format!("yaf_array_set({}, {}, {});", name, index_result, value_result));
            },
            
            Statement::If { condition, then_block, else_block } => {
                let cond_result = self.generate_expression(condition)?;
                self.emit_line(&format!("if (yaf_to_bool({})) {{", cond_result));
                self.indent();
                self.generate_block(then_block)?;
                self.dedent();
                
                if let Some(else_block) = else_block {
                    self.emit_line("} else {");
                    self.indent();
                    self.generate_block(else_block)?;
                    self.dedent();
                }
                
                self.emit_line("}");
            },
            
            Statement::While { condition, body } => {
                self.emit_line("while (1) {");
                self.indent();
                let cond_result = self.generate_expression(condition)?;
                self.emit_line(&format!("if (!yaf_to_bool({})) break;", cond_result));
                self.generate_block(body)?;
                self.dedent();
                self.emit_line("}");
            },
            
            Statement::For { init, condition, increment, body } => {
                // Generar la inicialización
                self.generate_statement(init)?;
                
                self.emit_line("while (1) {");
                self.indent();
                
                // Generar la condición
                let cond_result = self.generate_expression(condition)?;
                self.emit_line(&format!("if (!yaf_to_bool({})) break;", cond_result));
                
                // Generar el cuerpo
                self.generate_block(body)?;
                
                // Generar el incremento
                self.generate_statement(increment)?;
                
                self.dedent();
                self.emit_line("}");
            },
            
            Statement::Return { value } => {
                if let Some(expr) = value {
                    let expr_result = self.generate_expression(expr)?;
                    self.emit_line(&format!("return {};", expr_result));
                } else {
                    self.emit_line("return yaf_make_void();");
                }
            },
            
            Statement::Expression(expr) => {
                let result = self.generate_expression(expr)?;
                // Para expresiones como statements, emitir la línea si no es solo un literal
                match expr {
                    Expression::FunctionCall { .. } => {
                        // Las llamadas de función ya han emitido su código en generate_expression
                        // Solo necesitamos agregar el punto y coma si no es print
                        if let Expression::FunctionCall { name, .. } = expr {
                            if name != "print" {
                                self.emit_line(&format!("{};", result));
                            }
                        }
                    },
                    _ => {
                        // Otras expresiones como statements se descartan
                    }
                }
            },
        }
        Ok(())
    }
    
    fn generate_expression(&mut self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(value) => {
                match value {
                    Value::Int(n) => Ok(format!("yaf_make_int({})", n)),
                    Value::String(s) => {
                        // Escapar string para C
                        let escaped = s.replace("\\", "\\\\")
                                      .replace("\"", "\\\"")
                                      .replace("\n", "\\n")
                                      .replace("\t", "\\t");
                        Ok(format!("yaf_make_string(\"{}\")", escaped))
                    },
                    Value::Float(f) => Ok(format!("yaf_make_int({})", *f as i64)), // Temporal: convertir a int
                    Value::Bool(b) => Ok(format!("yaf_make_bool({})", if *b { "true" } else { "false" })),
                }
            },
            
            Expression::Variable(name) => Ok(name.clone()),
            
            Expression::FunctionCall { name, arguments } => {
                if name == "print" {
                    // La función print debe ser tratada como un statement, no como expresión
                    // Generar una función auxiliar temporal
                    let mut print_code = String::new();
                    
                    for (i, arg) in arguments.iter().enumerate() {
                        if i > 0 {
                            print_code.push_str("printf(\" \"); ");
                        }
                        let arg_result = self.generate_expression(arg)?;
                        print_code.push_str(&format!("yaf_print_value({}); ", arg_result));
                    }
                    print_code.push_str("printf(\"\\n\");");
                    
                    // Emitir el código del print
                    self.emit_line(&print_code);
                    Ok("yaf_make_void()".to_string())
                } else {
                    // Llamada a función de usuario
                    let mut call = format!("yaf_func_{}(", name);
                    for (i, arg) in arguments.iter().enumerate() {
                        if i > 0 {
                            call.push_str(", ");
                        }
                        let arg_result = self.generate_expression(arg)?;
                        call.push_str(&arg_result);
                    }
                    call.push(')');
                    Ok(call)
                }
            },
            
            Expression::BinaryOp { left, operator, right } => {
                let left_result = self.generate_expression(left)?;
                let right_result = self.generate_expression(right)?;
                
                let op_func = match operator {
                    BinaryOperator::Add => "yaf_add",
                    BinaryOperator::Subtract => "yaf_sub", 
                    BinaryOperator::Multiply => "yaf_mul",
                    BinaryOperator::Divide => "yaf_div",
                    BinaryOperator::Modulo => "yaf_mod",
                    BinaryOperator::Equal => "yaf_eq",
                    BinaryOperator::NotEqual => {
                        return Ok(format!("yaf_make_bool(!yaf_to_bool(yaf_eq({}, {})))", left_result, right_result));
                    },
                    BinaryOperator::Less => "yaf_lt",
                    BinaryOperator::LessEqual => "yaf_le",
                    BinaryOperator::Greater => "yaf_gt",
                    BinaryOperator::GreaterEqual => "yaf_ge",
                    BinaryOperator::And => {
                        return Ok(format!("yaf_make_bool(yaf_to_bool({}) && yaf_to_bool({}))", left_result, right_result));
                    },
                    BinaryOperator::Or => {
                        return Ok(format!("yaf_make_bool(yaf_to_bool({}) || yaf_to_bool({}))", left_result, right_result));
                    },
                };
                
                Ok(format!("{}({}, {})", op_func, left_result, right_result))
            },
            
            Expression::UnaryOp { operator, operand } => {
                let operand_result = self.generate_expression(operand)?;
                
                match operator {
                    UnaryOperator::Not => Ok(format!("yaf_make_bool(!yaf_to_bool({}))", operand_result)),
                    UnaryOperator::Minus => Ok(format!("yaf_sub(yaf_make_int(0), {})", operand_result)),
                }
            },
            
            Expression::ArrayLiteral { elements: _ } => {
                // Por simplicidad, todos los arrays son vacíos por ahora
                Ok("yaf_make_array()".to_string())
            },
            
            Expression::ArrayAccess { array: _, index: _ } => {
                // Por simplicidad, retornamos int 0 por ahora
                Ok("yaf_make_int(0)".to_string())
            },
            
            Expression::BuiltinCall { name, arguments } => {
                let mut c_args = Vec::new();
                for arg in arguments {
                    c_args.push(self.generate_expression(arg)?);
                }
                let args_str = c_args.join(", ");
                
                match name.as_str() {
                    // Math functions
                    "abs" => Ok(format!("yaf_math_abs({})", args_str)),
                    "max" => Ok(format!("yaf_math_max({})", args_str)),
                    "min" => Ok(format!("yaf_math_min({})", args_str)),
                    "pow" => Ok(format!("yaf_math_pow({})", args_str)),
                    
                    // String functions
                    "length" => Ok(format!("yaf_string_length({})", args_str)),
                    "upper" => Ok(format!("yaf_string_upper({})", args_str)),
                    "lower" => Ok(format!("yaf_string_lower({})", args_str)),
                    "concat" => Ok(format!("yaf_string_concat({})", args_str)),
                    
                    // I/O functions
                    "read_file" => Ok(format!("yaf_io_read_file({})", args_str)),
                    "write_file" => Ok(format!("yaf_io_write_file({})", args_str)),
                    "file_exists" => Ok(format!("yaf_io_file_exists({})", args_str)),
                    
                    // Time functions
                    "now" => Ok("yaf_time_now()".to_string()),
                    "now_millis" => Ok("yaf_time_now_millis()".to_string()),
                    "sleep" => Ok(format!("yaf_time_sleep({})", args_str)),
                    
                    // Type conversion functions
                    "str" => Ok(format!("yaf_value_to_string({})", args_str)),
                    "int" => Ok(format!("yaf_value_to_int({})", args_str)),
                    "float" => Ok(format!("yaf_value_to_float({})", args_str)),
                    
                    _ => Err(YafError::TypeError(format!("Unknown builtin function: {}", name)))
                }
            },
        }
    }
    
    fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }
    
    fn indent(&mut self) {
        self.indent_level += 1;
    }
    
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}