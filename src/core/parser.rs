use crate::core::ast::*;
use crate::core::lexer::{Token, TokenInfo};
use crate::runtime::values::Value;
use crate::error::{YafError, Result};

pub struct Parser {
    tokens: Vec<TokenInfo>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    fn current_token(&self) -> &Token {
        if let Some(token_info) = self.tokens.get(self.current) {
            &token_info.token
        } else {
            &Token::Eof
        }
    }
    
    fn current_position(&self) -> (usize, usize) {
        if let Some(token_info) = self.tokens.get(self.current) {
            (token_info.line, token_info.column)
        } else {
            (1, 1)
        }
    }
    
    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        self.current_token()
    }
    
    fn check(&self, token_type: &Token) -> bool {
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(token_type)
    }
    
    fn consume(&mut self, expected: Token, message: &str) -> Result<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            self.create_parse_error(&expected, message)
        }
    }
    
    fn create_parse_error<T>(&self, expected: &Token, context: &str) -> Result<T> {
        let current = self.current_token();
        let expected_desc = self.token_description(expected);
        let found_desc = self.token_description(current);
        
        let message = if context.is_empty() {
            format!("Se esperaba {}, encontrado: {}", expected_desc, found_desc)
        } else {
            format!("Se esperaba {} {}, encontrado: {}", expected_desc, context, found_desc)
        };
        
        // Añadir información de posición estimada basada en el token actual
        let position_info = format!("Posición del token: {} de {} (token #{}/{})", 
            self.current + 1, self.tokens.len(), self.current, self.tokens.len() - 1);
        
        // Mostrar contexto de tokens cercanos para debug
        let context_tokens = self.get_token_context();
        let context_info = if !context_tokens.is_empty() {
            format!("\nContexto de tokens: {}", context_tokens)
        } else {
            String::new()
        };
        
        // Agregar sugerencias contextuales
        let suggestion = self.get_suggestion_for_error(expected, current);
        let final_message = if let Some(hint) = suggestion {
            format!("{}\n{}{}\nSugerencia: {}", message, position_info, context_info, hint)
        } else {
            format!("{}\n{}{}", message, position_info, context_info)
        };
        
        let (line, column) = self.current_position();
        Err(YafError::ParseErrorWithPosition { 
            message: final_message, 
            token_index: self.current, 
            total_tokens: self.tokens.len(),
            line,
            column
        })
    }
    
    fn get_token_context(&self) -> String {
        let start = self.current.saturating_sub(2);
        let end = (self.current + 3).min(self.tokens.len());
        
        let mut context = Vec::new();
        for i in start..end {
            let marker = if i == self.current { ">>>" } else { "" };
            if let Some(token) = self.tokens.get(i) {
                context.push(format!("{}{:?}", marker, token));
            }
        }
        
        context.join(" ")
    }
    
    fn get_suggestion_for_error(&self, expected: &Token, found: &Token) -> Option<String> {
        match (expected, found) {
            (Token::Arrow, Token::LeftBrace) => {
                Some("En YAF, el tipo de retorno es opcional. Puedes usar directamente '{' después de los parámetros.".to_string())
            },
            (Token::LeftBrace, Token::Arrow) => {
                Some("Si especificas un tipo de retorno con '->', debe ir seguido del cuerpo de la función con '{'.".to_string())
            },
            (Token::RightParen, Token::LeftBrace) => {
                Some("Falta cerrar los parámetros con ')' antes del cuerpo de la función.".to_string())
            },
            (Token::RightParen, Token::Comma) => {
                Some("Error en los parámetros de función. Verifica la sintaxis: 'func nombre(param1: tipo1, param2: tipo2)'.".to_string())
            },
            (Token::Colon, Token::LeftBrace) => {
                Some("En YAF, las funciones van directamente de ')' a '{' sin necesidad de ':'.".to_string())
            },
            (Token::Semicolon, Token::Eof) => {
                Some("Falta ';' al final de la declaración.".to_string())
            },

            (Token::Identifier(_), Token::IntLiteral(_)) => {
                Some("Verifica que estés usando la sintaxis correcta para identificadores de variables o funciones.".to_string())
            },
            _ => None
        }
    }
    
    fn token_description(&self, token: &Token) -> String {
        match token {
            Token::Fun => "'func'".to_string(),
            Token::If => "'if'".to_string(),
            Token::Else => "'else'".to_string(),
            Token::While => "'while'".to_string(),
            Token::For => "'for'".to_string(),
            Token::Return => "'return'".to_string(),

            Token::Print => "'print'".to_string(),
            Token::LeftParen => "'('".to_string(),
            Token::RightParen => "')'".to_string(),
            Token::LeftBrace => "'{'".to_string(),
            Token::RightBrace => "'}'".to_string(),
            Token::LeftBracket => "'['".to_string(),
            Token::RightBracket => "']'".to_string(),
            Token::Arrow => "'->'".to_string(),
            Token::Semicolon => "';'".to_string(),
            Token::Comma => "','".to_string(),
            Token::Colon => "':'".to_string(),
            Token::Equal => "'='".to_string(),
            Token::Plus => "'+'".to_string(),
            Token::Minus => "'-'".to_string(),
            Token::Star => "'*'".to_string(),
            Token::Percent => "'%'".to_string(),
            Token::Slash => "'/'".to_string(),
            Token::EqualEqual => "'=='".to_string(),
            Token::NotEqual => "'!='".to_string(),
            Token::Less => "'<'".to_string(),
            Token::LessEqual => "'<='".to_string(),
            Token::Greater => "'>'".to_string(),
            Token::GreaterEqual => "'>='".to_string(),
            Token::And => "'&&'".to_string(),
            Token::Or => "'||'".to_string(),
            Token::Not => "'!'".to_string(),
            Token::Identifier(name) => format!("identificador '{}'", name),
            Token::IntLiteral(n) => format!("número entero {}", n),
            Token::FloatLiteral(n) => format!("número decimal {}", n),
            Token::StringLiteral(s) => format!("string \"{}\"", s),
            Token::BoolLiteral(b) => format!("booleano {}", b),
            Token::IntType => "'int'".to_string(),
            Token::FloatType => "'float'".to_string(),
            Token::StringType => "'string'".to_string(),
            Token::BoolType => "'bool'".to_string(),
            Token::VoidType => "'void'".to_string(),
            Token::ArrayType => "'array'".to_string(),
            Token::Eof => "fin de archivo".to_string(),
        }
    }
    
    pub fn parse(&mut self) -> Result<Program> {
        let mut functions = Vec::new();
        let mut main_statements = Vec::new();
        let mut main_function: Option<Function> = None;
        
        // Parsear funciones y declaraciones globales
        while !self.check(&Token::Eof) {
    
            if self.check(&Token::Fun) {
                let function = self.parse_function()?;
                
                // Verificar si es función main
                if function.name == "main" {
                    if main_function.is_some() {
                        return Err(YafError::ParseError("Solo puede haber una función main".to_string()));
                    }
                    
                    // Validar que main no tenga parámetros y tenga tipo de retorno void
                    if !function.parameters.is_empty() {
                        return Err(YafError::ParseError("La función main no puede tener parámetros".to_string()));
                    }
                    if function.return_type != Type::Void {
                        return Err(YafError::ParseError("La función main debe tener tipo de retorno void".to_string()));
                    }
                    
                    main_function = Some(function);
                } else {
                    functions.push(function);
                }
            } else {
                // Parsear declaraciones/asignaciones globales

                let stmt = self.parse_statement()?;
                main_statements.push(stmt);
            }
        }
        
        // Crear el bloque main
        let main = if let Some(func) = main_function {
            // Si hay función main, combinar declaraciones globales + función main
            let mut combined_statements = main_statements;
            combined_statements.extend(func.body.statements);
            Block { statements: combined_statements }
        } else if !main_statements.is_empty() {
            // Si no hay función main pero hay declaraciones globales, usarlas como main
            Block { statements: main_statements }
        } else {
            return Err(YafError::ParseError("Se requiere una función main o declaraciones globales".to_string()));
        };
        
        Ok(Program { functions, main })
    }
    
    fn parse_function(&mut self) -> Result<Function> {
        self.consume(Token::Fun, "Se esperaba 'func'")?;
        
        let name = match self.current_token() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => return Err(YafError::ParseError("Se esperaba nombre de función".to_string())),
        };

        self.consume(Token::LeftParen, "Se esperaba '(' después del nombre de función")?;
        
        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let param_name = match self.current_token() {
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        name
                    },
                    _ => return Err(YafError::ParseError("Se esperaba nombre de parámetro".to_string())),
                };
                
                self.consume(Token::Colon, "Se esperaba ':' después del nombre del parámetro")?;
                
                let param_type = self.parse_type()?;
                
                parameters.push(Parameter {
                    name: param_name,
                    param_type,
                });
                
                if self.check(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Se esperaba ')'")?;
        
        // El tipo de retorno es opcional para todas las funciones
        let return_type = if self.check(&Token::Arrow) {
            self.advance(); // consume ->
            let ret_type = self.parse_type()?;
            
            // Para main, verificar que sea void
            if name == "main" && ret_type != Type::Void {
                return Err(YafError::ParseError("La función main solo puede tener tipo de retorno void".to_string()));
            }
            
            ret_type
        } else {
            Type::Void // Por defecto, las funciones sin tipo de retorno explícito son void
        };
        
        // En YAF, las funciones van directamente de los parámetros al cuerpo sin ':' 
        // Solo consumir ':' si está presente para compatibilidad hacia atrás
        if self.check(&Token::Colon) {
            self.advance();
        }
        
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        match self.current_token() {
            Token::IntType => {
                self.advance();
                Ok(Type::Int)
            },
            Token::FloatType => {
                self.advance();
                Ok(Type::Float)
            },
            Token::StringType => {
                self.advance();
                Ok(Type::String)
            },
            Token::BoolType => {
                self.advance();
                Ok(Type::Bool)
            },
            Token::VoidType => {
                self.advance();
                Ok(Type::Void)
            },
            Token::ArrayType => {
                self.advance(); // consume 'array'
                self.consume(Token::LeftBracket, "Se esperaba '[' después de 'array'")?;
                let element_type = self.parse_type()?;
                self.consume(Token::RightBracket, "Se esperaba ']' después del tipo de elemento")?;
                Ok(Type::Array(Box::new(element_type)))
            },
            _ => Err(YafError::ParseError("Se esperaba un tipo".to_string())),
        }
    }
    
    fn parse_block(&mut self) -> Result<Block> {
        self.consume(Token::LeftBrace, "Se esperaba '{'")?;
        
        let mut statements = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.check(&Token::Eof) {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(Token::RightBrace, "Se esperaba '}'")?;
        
        Ok(Block { statements })
    }
    
    fn parse_statement(&mut self) -> Result<Statement> {
        let stmt = match self.current_token() {
            Token::If => self.parse_if_statement()?,
            Token::While => self.parse_while_statement()?,
            Token::For => self.parse_for_statement()?,
            Token::Return => {
                let ret_stmt = self.parse_return_statement()?;
                // Consumir semicolon opcional después de return
                if self.check(&Token::Semicolon) {
                    self.advance();
                }
                ret_stmt
            },
            Token::Identifier(_) => {
                // Mirar hacia adelante para determinar si es asignación o declaración
                let current_pos = self.current;
                
                // Avanzar para ver el siguiente token
                let name = if let Token::Identifier(name) = self.current_token() {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(YafError::ParseError("Expected identifier".to_string()));
                };
                
                // Determinar el tipo de statement basado en el siguiente token
                let stmt = match self.current_token() {
                    Token::Colon => {
                        // Es declaración: nombre: tipo = valor
                        self.advance(); // consume ':'
                        let var_type = self.parse_type()?;
                        
                        if !self.check(&Token::Equal) {
                            return Err(YafError::ParseError("Se esperaba '=' después del tipo en declaración de variable".to_string()));
                        }
                        self.advance(); // consume '='
                        
                        let value = self.parse_expression()?;
                        Statement::Declaration { name, var_type, value }
                    },
                    Token::Equal => {
                        // Es asignación: nombre = valor
                        self.advance(); // consume '='
                        let value = self.parse_expression()?;
                        Statement::Assignment { name, value }
                    },
                    Token::LeftBracket => {
                        // Es asignación a array: nombre[index] = valor
                        self.advance(); // consume '['
                        let index = self.parse_expression()?;
                        
                        if !self.check(&Token::RightBracket) {
                            return Err(YafError::ParseError("Se esperaba ']' después del índice".to_string()));
                        }
                        self.advance(); // consume ']'
                        
                        if !self.check(&Token::Equal) {
                            return Err(YafError::ParseError("Se esperaba '=' después de la expresión de array".to_string()));
                        }
                        self.advance(); // consume '='
                        
                        let value = self.parse_expression()?;
                        Statement::ArrayAssignment { name, index, value }
                    },
                    Token::LeftParen => {
                        // Es llamada de función: nombre(args...)
                        // Retroceder para parsear toda la expresión de llamada de función
                        self.current = current_pos;
                        let expr = self.parse_expression()?;
                        Statement::Expression(expr)
                    },
                    _ => {
                        // Es una expresión (variable u otra cosa)
                        // Retroceder para parsear toda la expresión
                        self.current = current_pos;
                        let expr = self.parse_expression()?;
                        Statement::Expression(expr)
                    }
                };
                
                // Consumir semicolon opcional
                if self.check(&Token::Semicolon) {
                    self.advance();
                }
                stmt
            },
            _ => {
                let expr = self.parse_expression()?;
                let stmt = Statement::Expression(expr);
                // Consumir semicolon opcional
                if self.check(&Token::Semicolon) {
                    self.advance();
                }
                stmt
            }
        };
        
        Ok(stmt)
    }
    
    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.consume(Token::If, "Se esperaba 'if'")?;
        let condition = self.parse_expression()?;
        // En YAF, no se requiere ':' después de la condición del if
        let then_block = self.parse_block()?;
        
        let else_block = if self.check(&Token::Else) {
            self.advance();
            // Tampoco se requiere ':' después de 'else'
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }
    
    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(Token::While, "Se esperaba 'while'")?;
        let condition = self.parse_expression()?;
        // En YAF, no se requiere ':' después de la condición del while
        let body = self.parse_block()?;
        
        Ok(Statement::While { condition, body })
    }
    
    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(Token::For, "Se esperaba 'for'")?;
        
        // Parsear la inicialización - solo asignación
        let init = if let Token::Identifier(_) = self.current_token() {
            // Asignación simple: i = 1
            let name = if let Token::Identifier(n) = self.current_token() {
                n.clone()
            } else {
                return Err(YafError::ParseError("Se esperaba identificador".to_string()));
            };
            self.advance();
            self.consume(Token::Equal, "Se esperaba '=' en la inicialización del for")?;
            let value = self.parse_expression()?;
            Box::new(Statement::Assignment { name, value })
        } else {
            return Err(YafError::ParseError("Se esperaba declaración o asignación en for".to_string()));
        };
        
        self.consume(Token::Semicolon, "Se esperaba ';' después de la inicialización del for")?;
        
        // Parsear la condición (ej: i <= 10)
        let condition = self.parse_expression()?;
        self.consume(Token::Semicolon, "Se esperaba ';' después de la condición del for")?;
        
        // Parsear el incremento - debe ser una asignación
        let increment = if let Token::Identifier(_) = self.current_token() {
            let name = if let Token::Identifier(n) = self.current_token() {
                n.clone()
            } else {
                return Err(YafError::ParseError("Se esperaba identificador".to_string()));
            };
            self.advance();
            self.consume(Token::Equal, "Se esperaba '=' en el incremento del for")?;
            let value = self.parse_expression()?;
            Box::new(Statement::Assignment { name, value })
        } else {
            return Err(YafError::ParseError("Se esperaba asignación en el incremento del for".to_string()));
        };
        
        // Parsear el cuerpo del loop
        let body = self.parse_block()?;
        
        Ok(Statement::For { init, condition, increment, body })
    }
    
    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.consume(Token::Return, "Se esperaba 'return'")?;
        
        let value = if self.check(&Token::LeftBrace) || self.check(&Token::Eof) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        Ok(Statement::Return { value })
    }
    

    

    

    
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;
        
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;
        
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;
        
        while matches!(self.current_token(), Token::EqualEqual | Token::NotEqual) {
            let operator = match self.current_token() {
                Token::EqualEqual => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;
        
        while matches!(self.current_token(), Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual) {
            let operator = match self.current_token() {
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;
        
        while matches!(self.current_token(), Token::Plus | Token::Minus) {
            let operator = match self.current_token() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;
        
        while matches!(self.current_token(), Token::Star | Token::Slash | Token::Percent) {
            let operator = match self.current_token() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expression> {
        match self.current_token() {
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            },
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(operand),
                })
            },
            _ => self.parse_postfix(),
        }
    }
    
    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        
        loop {
            match self.current_token() {
                Token::LeftParen => {
                    // Function call
                    if let Expression::Variable(name) = expr {
                        self.advance(); // consume (
                        let mut arguments = Vec::new();
                        if !self.check(&Token::RightParen) {
                            loop {
                                arguments.push(self.parse_expression()?);
                                if self.check(&Token::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                        self.consume(Token::RightParen, "Se esperaba ')' después de los argumentos")?;
                        expr = Expression::FunctionCall { name, arguments };
                    } else {
                        return Err(YafError::ParseError("Solo las variables pueden ser llamadas como funciones".to_string()));
                    }
                },
                Token::LeftBracket => {
                    // Array access
                    self.advance(); // consume [
                    let index = self.parse_expression()?;
                    self.consume(Token::RightBracket, "Se esperaba ']' después del índice del array")?;
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                },
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expression> {
        match self.current_token().clone() {
            Token::IntLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Value::Int(value)))
            },
            Token::FloatLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Value::Float(value)))
            },
            Token::StringLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Value::String(value)))
            },
            Token::BoolLiteral(value) => {
                self.advance();
                Ok(Expression::Literal(Value::Bool(value)))
            },
            Token::LeftBracket => {
                self.advance(); // consume [
                let mut elements = Vec::new();
                
                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                
                self.consume(Token::RightBracket, "Se esperaba ']' después de los elementos del array")?;
                Ok(Expression::ArrayLiteral { elements })
            },
            Token::Identifier(name) => {
                let name = name.clone();
                
                // Verificar si es una función de librería built-in
                if matches!(name.as_str(), "abs" | "max" | "min" | "pow" | "length" | "upper" | "lower" | "concat" | "substring" | "read_file" | "write_file" | "file_exists" | "now" | "now_millis" | "sleep" | "str" | "int" | "float" | "input" | "input_prompt" | "string_to_int" | "int_to_string") {
                    self.advance();
                    self.consume(Token::LeftParen, "Se esperaba '(' después de función built-in")?;
                    
                    let mut arguments = Vec::new();
                    if !self.check(&Token::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if self.check(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    
                    self.consume(Token::RightParen, "Se esperaba ')'")?;
                    Ok(Expression::BuiltinCall { name, arguments })
                } else {
                    self.advance(); // consume identifier
                    
                    if self.check(&Token::LeftParen) {
                        // Llamada a función definida por usuario
                        self.advance(); // consume '('
                        
                        let mut arguments = Vec::new();
                        if !self.check(&Token::RightParen) {
                            loop {
        
                                arguments.push(self.parse_expression()?);
        
                                if self.check(&Token::Comma) {
        
                                    self.advance();
        
                                } else {
        
                                    break;
                                }
                            }
                        }
                        

                        self.consume(Token::RightParen, "Se esperaba ')'")?;
                        Ok(Expression::FunctionCall { name, arguments })
                    } else {
                        // Variable
                        Ok(Expression::Variable(name))
                    }
                }
            },
            Token::Print => {
                self.advance();
                self.consume(Token::LeftParen, "Se esperaba '(' después de 'print'")?;
                
                let mut arguments = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                
                self.consume(Token::RightParen, "Se esperaba ')'")?;
                
                Ok(Expression::FunctionCall {
                    name: "print".to_string(),
                    arguments,
                })
            },
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Se esperaba ')'")?;
                Ok(expr)
            },
            _ => {
                let current = self.current_token();
                let message = format!("Token inesperado: {:?}", current);
                
                let (line, column) = self.current_position();
                Err(YafError::ParseErrorWithPosition { 
                    message, 
                    token_index: self.current, 
                    total_tokens: self.tokens.len(),
                    line,
                    column
                })
            }
        }
    }
}