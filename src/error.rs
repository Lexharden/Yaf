#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum YafError {
    LexError(String),
    LexErrorWithPosition { message: String, line: usize, column: usize },
    ParseError(String),
    ParseErrorWithPosition { message: String, token_index: usize, total_tokens: usize, line: usize, column: usize },
    TypeError(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    ArgumentMismatch(String),
    IoError(String),
    RuntimeError(String),
    ValueError(String),
    IndexError(String),
}

impl std::fmt::Display for YafError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YafError::LexError(msg) => write!(f, "Lexical error: {}", msg),
            YafError::LexErrorWithPosition { message, line, column } => 
                write!(f, "Lexical error: {} (line {}:{})", message, line, column),
            YafError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            YafError::ParseErrorWithPosition { message, token_index, total_tokens, line, column } => 
                write!(f, "Parse error: {} (line {}:{}, token {}/{})", message, line, column, token_index + 1, total_tokens),
            YafError::TypeError(msg) => write!(f, "Type error: {}", msg),
            YafError::UndefinedVariable(msg) => write!(f, "Undefined variable: {}", msg),
            YafError::UndefinedFunction(msg) => write!(f, "Undefined function: {}", msg),
            YafError::ArgumentMismatch(msg) => write!(f, "Argument mismatch: {}", msg),
            YafError::IoError(msg) => write!(f, "IO error: {}", msg),
            YafError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            YafError::ValueError(msg) => write!(f, "Value error: {}", msg),
            YafError::IndexError(msg) => write!(f, "Index error: {}", msg),
        }
    }
}

impl std::error::Error for YafError {}

impl From<std::io::Error> for YafError {
    fn from(err: std::io::Error) -> Self {
        YafError::IoError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, YafError>;