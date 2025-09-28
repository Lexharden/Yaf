use std::collections::HashMap;
use crate::core::ast::*;
use crate::runtime::values::Value;
use crate::error::{YafError, Result};

pub struct TypeChecker {
    variables: HashMap<String, Type>,
    functions: HashMap<String, (Vec<Type>, Type)>, // (param_types, return_type)
    current_function_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            variables: HashMap::new(),
            functions: HashMap::new(),
            current_function_return_type: None,
        }
    }
    
    pub fn check(&mut self, program: &Program) -> Result<()> {
        // First pass: collect function signatures
        for function in &program.functions {
            let param_types: Vec<Type> = function.parameters
                .iter()
                .map(|p| p.param_type.clone())
                .collect();
            
            self.functions.insert(
                function.name.clone(),
                (param_types, function.return_type.clone())
            );
        }
        
        // Second pass: Collect global variables from main block (only let statements)
        self.current_function_return_type = Some(Type::Void);
        self.variables.clear();
        self.collect_global_variables(&program.main)?;
        
        // Third pass: type check function bodies with global variables available
        for function in &program.functions {
            self.check_function(function)?;
        }
        
        // Finally: type check main block completely
        self.current_function_return_type = Some(Type::Void);
        self.check_block(&program.main)?;
        
        Ok(())
    }
    
    fn collect_global_variables(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            match statement {
                Statement::Assignment { name, value } => {
                    let value_type = self.check_expression(value)?;
                    self.variables.insert(name.clone(), value_type);
                },
                _ => {} // Ignore other statements for global variable collection
            }
        }
        Ok(())
    }

    fn check_function(&mut self, function: &Function) -> Result<()> {
        // Keep global variables and add parameters to scope
        let mut function_scope = self.variables.clone();
        
        self.current_function_return_type = Some(function.return_type.clone());
        
        // Add parameters to scope
        for param in &function.parameters {
            function_scope.insert(param.name.clone(), param.param_type.clone());
        }
        
        // Temporarily use function scope
        let original_variables = std::mem::replace(&mut self.variables, function_scope);
        
        self.check_block(&function.body)?;
        
        // Restore global variables
        self.variables = original_variables;
        Ok(())
    }
    
    fn check_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }
    
    fn check_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Declaration { name, var_type, value } => {
                let value_type = self.check_expression(value)?;
                
                // Verificar que el tipo del valor coincida con el tipo declarado
                if value_type != *var_type {
                    return Err(YafError::TypeError(format!(
                        "Cannot assign {} to variable '{}' of declared type {}",
                        value_type.to_string(), name, var_type.to_string()
                    )));
                }
                
                // Verificar que la variable no exista ya
                if self.variables.contains_key(name) {
                    return Err(YafError::TypeError(format!(
                        "Variable '{}' is already declared", name
                    )));
                }
                
                // Registrar la variable con su tipo
                self.variables.insert(name.clone(), var_type.clone());
            },
            Statement::Assignment { name, value } => {
                let value_type = self.check_expression(value)?;
                
                if let Some(existing_type) = self.variables.get(name) {
                    if *existing_type != value_type {
                        return Err(YafError::TypeError(format!(
                            "Cannot assign {} to variable '{}' of type {}",
                            value_type.to_string(), name, existing_type.to_string()
                        )));
                    }
                } else {
                    // Implicit declaration with inferred type
                    self.variables.insert(name.clone(), value_type);
                }
            },
            
            Statement::ArrayAssignment { name, index, value } => {
                let index_type = self.check_expression(index)?;
                let value_type = self.check_expression(value)?;
                
                // Verificar que el índice sea entero
                if index_type != Type::Int {
                    return Err(YafError::TypeError(format!(
                        "Array index must be integer, got {}", index_type.to_string()
                    )));
                }
                
                // Verificar que la variable existe y es un array
                if let Some(existing_type) = self.variables.get(name) {
                    match existing_type {
                        Type::Array(element_type) => {
                            if **element_type != value_type {
                                return Err(YafError::TypeError(format!(
                                    "Cannot assign {} to array '{}' of type {}",
                                    value_type.to_string(), name, element_type.to_string()
                                )));
                            }
                        },
                        _ => {
                            return Err(YafError::TypeError(format!(
                                "Variable '{}' is not an array", name
                            )));
                        }
                    }
                } else {
                    return Err(YafError::TypeError(format!(
                        "Variable '{}' not found", name
                    )));
                }
            },
            
            Statement::If { condition, then_block, else_block } => {
                let condition_type = self.check_expression(condition)?;
                if condition_type != Type::Bool && condition_type != Type::Int {
                    return Err(YafError::TypeError(format!(
                        "If condition must be bool or int, found {}",
                        condition_type.to_string()
                    )));
                }
                
                // Create new scope for blocks
                let saved_vars = self.variables.clone();
                self.check_block(then_block)?;
                
                if let Some(else_block) = else_block {
                    self.variables = saved_vars.clone();
                    self.check_block(else_block)?;
                }
                
                self.variables = saved_vars;
            },
            
            Statement::While { condition, body } => {
                let condition_type = self.check_expression(condition)?;
                if condition_type != Type::Bool && condition_type != Type::Int {
                    return Err(YafError::TypeError(format!(
                        "While condition must be bool or int, found {}",
                        condition_type.to_string()
                    )));
                }
                
                let saved_vars = self.variables.clone();
                self.check_block(body)?;
                self.variables = saved_vars;
            },
            
            Statement::For { init, condition, increment, body } => {
                // Crear nuevo scope para el loop for
                let saved_vars = self.variables.clone();
                
                // Verificar la inicialización
                self.check_statement(init)?;
                
                // Verificar la condición
                let condition_type = self.check_expression(condition)?;
                if condition_type != Type::Bool && condition_type != Type::Int {
                    return Err(YafError::TypeError(format!(
                        "For condition must be bool or int, found {}",
                        condition_type.to_string()
                    )));
                }
                
                // Verificar el incremento
                self.check_statement(increment)?;
                
                // Verificar el cuerpo
                self.check_block(body)?;
                
                // Restaurar el scope anterior
                self.variables = saved_vars;
            },
            
            Statement::Return { value } => {
                let return_type = if let Some(expr) = value {
                    self.check_expression(expr)?
                } else {
                    Type::Void
                };
                
                if let Some(expected_type) = &self.current_function_return_type {
                    if return_type != *expected_type {
                        return Err(YafError::TypeError(format!(
                            "Return type {} doesn't match function return type {}",
                            return_type.to_string(), expected_type.to_string()
                        )));
                    }
                }
            },
            
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
            },
        }
        Ok(())
    }
    
    fn check_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(value) => {
                Ok(match value {
                    Value::Int(_) => Type::Int,
                    Value::Float(_) => Type::Float,
                    Value::String(_) => Type::String,
                    Value::Bool(_) => Type::Bool,
                })
            },
            
            Expression::Variable(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| YafError::UndefinedVariable(name.clone()))
            },
            
            Expression::FunctionCall { name, arguments } => {
                // Check built-in functions first
                match name.as_str() {
                    // Basic functions
                    "print" => {
                        // Print accepts any type
                        for arg in arguments {
                            self.check_expression(arg)?;
                        }
                        return Ok(Type::Void);
                    },
                    
                    // Input functions
                    "input" => {
                        if !arguments.is_empty() {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'input' expects no arguments, got {}",
                                arguments.len()
                            )));
                        }
                        return Ok(Type::String);
                    },
                    "input_prompt" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'input_prompt' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'input_prompt' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        return Ok(Type::String);
                    },
                    
                    // Type conversion functions
                    "string_to_int" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'string_to_int' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'string_to_int' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        return Ok(Type::Int);
                    },
                    "int_to_string" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'int_to_string' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::Int {
                            return Err(YafError::TypeError(format!(
                                "Function 'int_to_string' expects int argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        return Ok(Type::String);
                    },
                    
                    // String functions
                    "string_length" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'string_length' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'string_length' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        return Ok(Type::Int);
                    },
                    "string_upper" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'string_upper' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'string_upper' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        return Ok(Type::String);
                    },
                    "string_lower" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'string_lower' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'string_lower' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        return Ok(Type::String);
                    },
                    _ => {
                        // Not a built-in function, continue to user-defined functions
                    }
                }
                
                if let Some((param_types, return_type)) = self.functions.get(name).cloned() {
                    if arguments.len() != param_types.len() {
                        return Err(YafError::ArgumentMismatch(format!(
                            "Function '{}' expects {} arguments, got {}",
                            name, param_types.len(), arguments.len()
                        )));
                    }
                    
                    for (i, (arg, expected_type)) in arguments.iter().zip(param_types.iter()).enumerate() {
                        let arg_type = self.check_expression(arg)?;
                        if arg_type != *expected_type {
                            return Err(YafError::TypeError(format!(
                                "Function '{}' argument {} expects {}, got {}",
                                name, i + 1, expected_type.to_string(), arg_type.to_string()
                            )));
                        }
                    }
                    
                    Ok(return_type)
                } else {
                    Err(YafError::UndefinedFunction(name.clone()))
                }
            },
            
            Expression::BinaryOp { left, operator, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                
                match operator {
                    BinaryOperator::Add => {
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) => Ok(Type::Int),
                            (Type::Float, Type::Float) => Ok(Type::Float),
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                            (Type::String, Type::String) => Ok(Type::String),
                            _ => Err(YafError::TypeError(format!(
                                "Cannot add {} and {}",
                                left_type.to_string(), right_type.to_string()
                            )))
                        }
                    },
                    
                    BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) => Ok(Type::Int),
                            (Type::Float, Type::Float) => Ok(Type::Float),
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                            _ => Err(YafError::TypeError(format!(
                                "Arithmetic operation requires numeric types, got {} and {}",
                                left_type.to_string(), right_type.to_string()
                            )))
                        }
                    },
                    
                    BinaryOperator::Modulo => {
                        if left_type == Type::Int && right_type == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(YafError::TypeError(format!(
                                "Modulo operation requires int types, got {} and {}",
                                left_type.to_string(), right_type.to_string()
                            )))
                        }
                    },
                    
                    BinaryOperator::Equal | BinaryOperator::NotEqual => {
                        if left_type == right_type {
                            Ok(Type::Bool)
                        } else {
                            Err(YafError::TypeError(format!(
                                "Cannot compare {} and {}",
                                left_type.to_string(), right_type.to_string()
                            )))
                        }
                    },
                    
                    BinaryOperator::Less | BinaryOperator::LessEqual |
                    BinaryOperator::Greater | BinaryOperator::GreaterEqual => {
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) => Ok(Type::Bool),
                            (Type::Float, Type::Float) => Ok(Type::Bool),
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Bool),
                            _ => Err(YafError::TypeError(format!(
                                "Comparison requires numeric types, got {} and {}",
                                left_type.to_string(), right_type.to_string()
                            )))
                        }
                    },
                    
                    BinaryOperator::And | BinaryOperator::Or => {
                        // Allow bool or int (truthy values)
                        if (left_type == Type::Bool || left_type == Type::Int) &&
                           (right_type == Type::Bool || right_type == Type::Int) {
                            Ok(Type::Bool)
                        } else {
                            Err(YafError::TypeError(format!(
                                "Logic operation requires bool or int types, got {} and {}",
                                left_type.to_string(), right_type.to_string()
                            )))
                        }
                    },
                }
            },
            
            Expression::UnaryOp { operator, operand } => {
                let operand_type = self.check_expression(operand)?;
                
                match operator {
                    UnaryOperator::Not => {
                        if operand_type == Type::Bool || operand_type == Type::Int {
                            Ok(Type::Bool)
                        } else {
                            Err(YafError::TypeError(format!(
                                "Not operation requires bool or int, got {}",
                                operand_type.to_string()
                            )))
                        }
                    },
                    
                    UnaryOperator::Minus => {
                        if operand_type == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(YafError::TypeError(format!(
                                "Unary minus requires int, got {}",
                                operand_type.to_string()
                            )))
                        }
                    },
                }
            },
            
            Expression::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    // Array vacío - asumimos int por defecto
                    Ok(Type::Array(Box::new(Type::Int)))
                } else {
                    // Verificar que todos los elementos tengan el mismo tipo
                    let first_type = self.check_expression(&elements[0])?;
                    for (i, element) in elements.iter().enumerate().skip(1) {
                        let element_type = self.check_expression(element)?;
                        if element_type != first_type {
                            return Err(YafError::TypeError(format!(
                                "Array element {} has type {}, expected {}",
                                i, element_type.to_string(), first_type.to_string()
                            )));
                        }
                    }
                    Ok(Type::Array(Box::new(first_type)))
                }
            },
            
            Expression::ArrayAccess { array, index } => {
                let array_type = self.check_expression(array)?;
                let index_type = self.check_expression(index)?;
                
                if index_type != Type::Int {
                    return Err(YafError::TypeError(format!(
                        "Array index must be int, got {}",
                        index_type.to_string()
                    )));
                }
                
                match array_type {
                    Type::Array(element_type) => Ok(*element_type),
                    _ => Err(YafError::TypeError(format!(
                        "Cannot index non-array type {}",
                        array_type.to_string()
                    )))
                }
            },
            
            Expression::BuiltinCall { name, arguments } => {
                match name.as_str() {
                    // Basic functions
                    "print" => {
                        // Print accepts any type
                        for arg in arguments {
                            self.check_expression(arg)?;
                        }
                        Ok(Type::Void)
                    },
                    
                    // Math functions
                    "abs" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "abs() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::Int {
                            return Err(YafError::TypeError(format!(
                                "abs() expects int argument, got {}", arg_type.to_string()
                            )));
                        }
                        Ok(Type::Int)
                    },
                    "max" | "min" => {
                        if arguments.len() != 2 {
                            return Err(YafError::TypeError(format!(
                                "{}() expects 2 arguments, got {}", name, arguments.len()
                            )));
                        }
                        let arg1_type = self.check_expression(&arguments[0])?;
                        let arg2_type = self.check_expression(&arguments[1])?;
                        if arg1_type != Type::Int || arg2_type != Type::Int {
                            return Err(YafError::TypeError(format!(
                                "{}() expects int arguments, got {} and {}", 
                                name, arg1_type.to_string(), arg2_type.to_string()
                            )));
                        }
                        Ok(Type::Int)
                    },
                    "pow" => {
                        if arguments.len() != 2 {
                            return Err(YafError::TypeError(format!(
                                "pow() expects 2 arguments, got {}", arguments.len()
                            )));
                        }
                        let base_type = self.check_expression(&arguments[0])?;
                        let exp_type = self.check_expression(&arguments[1])?;
                        if base_type != Type::Int || exp_type != Type::Int {
                            return Err(YafError::TypeError(format!(
                                "pow() expects int arguments, got {} and {}", 
                                base_type.to_string(), exp_type.to_string()
                            )));
                        }
                        Ok(Type::Int)
                    },
                    
                    // Input functions
                    "input" => {
                        if !arguments.is_empty() {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'input' expects no arguments, got {}",
                                arguments.len()
                            )));
                        }
                        Ok(Type::String)
                    },
                    "input_prompt" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'input_prompt' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'input_prompt' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        Ok(Type::String)
                    },
                    
                    // Type conversion functions
                    "string_to_int" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'string_to_int' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "Function 'string_to_int' expects string argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        Ok(Type::Int)
                    },
                    "int_to_string" => {
                        if arguments.len() != 1 {
                            return Err(YafError::ArgumentMismatch(format!(
                                "Function 'int_to_string' expects 1 argument, got {}",
                                arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::Int {
                            return Err(YafError::TypeError(format!(
                                "Function 'int_to_string' expects int argument, got {}",
                                arg_type.to_string()
                            )));
                        }
                        Ok(Type::String)
                    },
                    
                    // String functions
                    "length" | "string_length" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "{}() expects 1 argument, got {}", name, arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "{}() expects string argument, got {}", name, arg_type.to_string()
                            )));
                        }
                        Ok(Type::Int)
                    },
                    "upper" | "lower" | "string_upper" | "string_lower" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "{}() expects 1 argument, got {}", name, arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "{}() expects string argument, got {}", name, arg_type.to_string()
                            )));
                        }
                        Ok(Type::String)
                    },
                    "concat" => {
                        if arguments.len() != 2 {
                            return Err(YafError::TypeError(format!(
                                "concat() expects 2 arguments, got {}", arguments.len()
                            )));
                        }
                        let arg1_type = self.check_expression(&arguments[0])?;
                        let arg2_type = self.check_expression(&arguments[1])?;
                        if arg1_type != Type::String || arg2_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "concat() expects string arguments, got {} and {}", 
                                arg1_type.to_string(), arg2_type.to_string()
                            )));
                        }
                        Ok(Type::String)
                    },
                    
                    // I/O functions
                    "read_file" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "read_file() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "read_file() expects string argument, got {}", arg_type.to_string()
                            )));
                        }
                        Ok(Type::String)
                    },
                    "write_file" => {
                        if arguments.len() != 2 {
                            return Err(YafError::TypeError(format!(
                                "write_file() expects 2 arguments, got {}", arguments.len()
                            )));
                        }
                        let path_type = self.check_expression(&arguments[0])?;
                        let content_type = self.check_expression(&arguments[1])?;
                        if path_type != Type::String || content_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "write_file() expects string arguments, got {} and {}", 
                                path_type.to_string(), content_type.to_string()
                            )));
                        }
                        Ok(Type::Bool)
                    },
                    "file_exists" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "file_exists() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::String {
                            return Err(YafError::TypeError(format!(
                                "file_exists() expects string argument, got {}", arg_type.to_string()
                            )));
                        }
                        Ok(Type::Bool)
                    },
                    
                    // Time functions
                    "now" | "now_millis" => {
                        if !arguments.is_empty() {
                            return Err(YafError::TypeError(format!(
                                "{}() expects no arguments, got {}", name, arguments.len()
                            )));
                        }
                        Ok(Type::Int)
                    },
                    "sleep" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "sleep() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let arg_type = self.check_expression(&arguments[0])?;
                        if arg_type != Type::Int {
                            return Err(YafError::TypeError(format!(
                                "sleep() expects int argument, got {}", arg_type.to_string()
                            )));
                        }
                        Ok(Type::Bool)
                    },
                    
                    // Type conversion functions
                    "str" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "str() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let _arg_type = self.check_expression(&arguments[0])?;
                        // str() puede convertir cualquier tipo a string
                        Ok(Type::String)
                    },
                    "int" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "int() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let _arg_type = self.check_expression(&arguments[0])?;
                        // int() puede convertir números y strings a int
                        Ok(Type::Int)
                    },
                    "float" => {
                        if arguments.len() != 1 {
                            return Err(YafError::TypeError(format!(
                                "float() expects 1 argument, got {}", arguments.len()
                            )));
                        }
                        let _arg_type = self.check_expression(&arguments[0])?;
                        // float() puede convertir números y strings a float
                        Ok(Type::Float)
                    },
                    
                    _ => Err(YafError::TypeError(format!("Unknown builtin function: {}", name)))
                }
            },
        }
    }
}