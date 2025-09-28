use crate::error::{YafError, Result};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize, // Longitud del token
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position { line, column, offset, length: 1 }
    }
    
    pub fn with_length(line: usize, column: usize, offset: usize, length: usize) -> Self {
        Position { line, column, offset, length }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct TokenWithPos {
    pub token: Token,
    pub position: Position,
    pub text: String, // Texto original del token
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

impl TokenInfo {
    pub fn new(token: Token, line: usize, column: usize) -> Self {
        TokenInfo { token, line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Palabras clave
    Fun,
    If,
    Else,
    While,
    For,
    Return,
    Print,
    
    // Tipos
    IntType,
    FloatType,
    StringType,
    BoolType,
    VoidType,
    ArrayType,
    
    // Literales
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    
    // Identificadores
    Identifier(String),
    
    // Operadores
    Plus,
    Minus,
    Star,
    Slash,
    Percent, // Operador módulo %
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Arrow,         // ->
    And,
    Or,
    Not,
    
    // Delimitadores
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,   // [
    RightBracket,  // ]
    Comma,
    Colon,
    Semicolon,
    
    // Comentarios y espacios en blanco se ignoran
    
    // Fin de archivo
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        
        Lexer {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
        }
    }
    
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    #[allow(dead_code)]
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.position)
    }
    
    #[allow(dead_code)]
    pub fn position_with_length(&self, start_pos: usize, start_line: usize, start_col: usize) -> Position {
        let length = self.position.saturating_sub(start_pos);
        Position::with_length(start_line, start_col, start_pos, length)
    }
    
    #[allow(dead_code)]
    pub fn create_token_with_pos(&self, token: Token, start_pos: usize, start_line: usize, start_col: usize, text: String) -> TokenWithPos {
        let position = self.position_with_length(start_pos, start_line, start_col);
        TokenWithPos { token, position, text }
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        // Saltar comentarios que empiezan con #
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_line_comment(&mut self) {
        // Saltar comentarios que empiezan con //
        self.advance(); // Saltar el primer /
        self.advance(); // Saltar el segundo /
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) -> Result<()> {
        // Saltar comentarios en bloque /* ... */
        self.advance(); // Saltar /
        self.advance(); // Saltar *
        
        while let Some(ch) = self.current_char {
            if ch == '*' && self.peek() == Some('/') {
                self.advance(); // Saltar *
                self.advance(); // Saltar /
                return Ok(());
            }
            self.advance();
        }
        
        Err(YafError::LexError("Comentario en bloque no cerrado".to_string()))
    }
    
    fn read_number(&mut self) -> Result<Token> {
        let mut number = String::new();
        let mut has_dot = false;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                // Solo permitir un punto decimal
                has_dot = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        if has_dot {
            number.parse::<f64>()
                .map(Token::FloatLiteral)
                .map_err(|_| YafError::LexError(format!("Número decimal inválido: {}", number)))
        } else {
            number.parse::<i64>()
                .map(Token::IntLiteral)
                .map_err(|_| YafError::LexError(format!("Número entero inválido: {}", number)))
        }
    }
    
    fn read_string(&mut self) -> Result<String> {
        let mut string = String::new();
        let start_line = self.line;
        let start_column = self.column;
        self.advance(); // Saltar la comilla inicial
        
        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // Saltar la comilla final
                return Ok(string);
            } else if ch == '\n' || ch == '\r' {
                // String que abarca múltiples líneas sin escape - probablemente error
                return Err(YafError::LexErrorWithPosition { 
                    message: "String no tiene comilla de cierre '\"' (string abarca múltiples líneas)".to_string(),
                    line: start_line,
                    column: start_column
                });
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some(c) => string.push(c),
                    None => return Err(YafError::LexErrorWithPosition { 
                        message: "String no tiene comilla de cierre '\"'".to_string(),
                        line: start_line,
                        column: start_column
                    }),
                }
                self.advance();
            } else {
                string.push(ch);
                self.advance();
            }
        }
        
        Err(YafError::LexErrorWithPosition { 
            message: "String no tiene comilla de cierre '\"'".to_string(),
            line: start_line,
            column: start_column
        })
    }
    
    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        identifier
    }
    
    fn identifier_to_keyword(&self, identifier: &str) -> Token {
        match identifier {
            "func" => Token::Fun,
            "function" => Token::Fun,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "return" => Token::Return,
            "print" => Token::Print,

            "int" => Token::IntType,
            "float" => Token::FloatType,
            "string" => Token::StringType,
            "bool" => Token::BoolType,
            "void" => Token::VoidType,
            "array" => Token::ArrayType,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            _ => Token::Identifier(identifier.to_string()),
        }
    }
    
    pub fn next_token(&mut self) -> Result<Token> {
        loop {
            match self.current_char {
                None => return Ok(Token::Eof),
                
                Some(ch) if ch.is_whitespace() => {
                    self.skip_whitespace();
                    continue;
                }
                
                Some('#') => {
                    self.skip_comment();
                    continue;
                }
                
                Some(ch) if ch.is_ascii_digit() => {
                    let token = self.read_number()?;
                    return Ok(token);
                }
                
                Some('"') => {
                    let string = self.read_string()?;
                    return Ok(Token::StringLiteral(string));
                }
                
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let identifier = self.read_identifier();
                    return Ok(self.identifier_to_keyword(&identifier));
                }
                
                Some('+') => {
                    self.advance();
                    return Ok(Token::Plus);
                }
                
                Some('-') => {
                    if self.peek() == Some('>') {
                        self.advance();
                        self.advance();
                        return Ok(Token::Arrow);
                    } else {
                        self.advance();
                        return Ok(Token::Minus);
                    }
                }
                
                Some('*') => {
                    self.advance();
                    return Ok(Token::Star);
                }
                
                Some('%') => {
                    self.advance();
                    return Ok(Token::Percent);
                }
                
                Some('/') => {
                    if self.peek() == Some('/') {
                        self.skip_line_comment();
                        continue;
                    } else if self.peek() == Some('*') {
                        self.skip_block_comment()?;
                        continue;
                    } else {
                        self.advance();
                        return Ok(Token::Slash);
                    }
                }
                
                Some('=') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Ok(Token::EqualEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Equal);
                    }
                }
                
                Some('!') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Ok(Token::NotEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Not);
                    }
                }
                
                Some('<') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Ok(Token::LessEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Less);
                    }
                }
                
                Some('>') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Ok(Token::GreaterEqual);
                    } else {
                        self.advance();
                        return Ok(Token::Greater);
                    }
                }
                
                Some('&') => {
                    if self.peek() == Some('&') {
                        self.advance();
                        self.advance();
                        return Ok(Token::And);
                    } else {
                        self.advance();
                        return Err(YafError::LexError("Carácter inesperado: &".to_string()));
                    }
                }
                
                Some('|') => {
                    if self.peek() == Some('|') {
                        self.advance();
                        self.advance();
                        return Ok(Token::Or);
                    } else {
                        self.advance();
                        return Err(YafError::LexError("Carácter inesperado: |".to_string()));
                    }
                }
                
                Some('(') => {
                    self.advance();
                    return Ok(Token::LeftParen);
                }
                
                Some(')') => {
                    self.advance();
                    return Ok(Token::RightParen);
                }
                
                Some('{') => {
                    self.advance();
                    return Ok(Token::LeftBrace);
                }
                
                Some('}') => {
                    self.advance();
                    return Ok(Token::RightBrace);
                }
                
                Some('[') => {
                    self.advance();
                    return Ok(Token::LeftBracket);
                }
                
                Some(']') => {
                    self.advance();
                    return Ok(Token::RightBracket);
                }
                
                Some(',') => {
                    self.advance();
                    return Ok(Token::Comma);
                }
                
                Some(':') => {
                    self.advance();
                    return Ok(Token::Colon);
                }
                
                Some(';') => {
                    self.advance();
                    return Ok(Token::Semicolon);
                }
                
                Some(ch) => {
                    self.advance();
                    return Err(YafError::LexError(format!("Carácter inesperado: {}", ch)));
                }
            }
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<TokenInfo>> {
        let mut tokens = Vec::new();
        
        loop {
            let current_line = self.line;
            let current_column = self.column;
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::Eof);
            
            tokens.push(TokenInfo::new(token, current_line, current_column));
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
}