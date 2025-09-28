use std::collections::HashMap;
use colored::*;
use crate::error::YafError;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub source_text: Option<String>, // Texto específico que causó el error
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub error: YafError,
    pub location: SourceLocation,
    pub suggestion: Option<String>, // Sugerencia de corrección
}

pub struct DiagnosticEngine {
    source_files: HashMap<String, String>,
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        DiagnosticEngine {
            source_files: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }
    
    pub fn add_source(&mut self, filename: &str, content: String) {
        self.source_files.insert(filename.to_string(), content);
    }
    
    #[allow(dead_code)]
    pub fn add_diagnostic(&mut self, error: YafError, filename: &str, line: usize, column: usize, length: usize, source_text: Option<String>, suggestion: Option<String>) {
        let diagnostic = Diagnostic {
            error,
            location: SourceLocation {
                file: filename.to_string(),
                line,
                column,
                length,
                source_text,
            },
            suggestion,
        };
        self.diagnostics.push(diagnostic);
    }
    
    pub fn from_yaf_error(&self, error: &YafError, filename: &str, line: usize, column: usize) {
        let location = Some(SourceLocation {
            file: filename.to_string(),
            line,
            column,
            length: 1,
            source_text: None,
        });
        self.print_error(error, location);
    }
    
    pub fn emit_all(&self) -> bool {
        for diagnostic in &self.diagnostics {
            self.print_diagnostic(diagnostic);
        }
        !self.diagnostics.is_empty()
    }
    
    #[allow(dead_code)]
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
    
    pub fn print_diagnostic(&self, diagnostic: &Diagnostic) {
        self.print_error(&diagnostic.error, Some(diagnostic.location.clone()));
        
        // Mostrar sugerencia si existe
        if let Some(suggestion) = &diagnostic.suggestion {
            eprintln!("{} {}", "help:".bright_cyan(), suggestion);
            eprintln!();
        }
    }
    
    pub fn print_error(&self, error: &YafError, location: Option<SourceLocation>) {
        let message = match error {
            YafError::LexError(msg) => format!("Lexical error: {}", msg),
            YafError::LexErrorWithPosition { message, line, column } => 
                format!("Lexical error: {} (line {}:{})", message, line, column),
            YafError::ParseError(msg) => format!("Parse error: {}", msg),
            YafError::ParseErrorWithPosition { message, token_index, total_tokens, line, column } => 
                format!("Parse error: {} (line {}:{}, token {}/{})", message, line, column, token_index + 1, total_tokens),
            YafError::TypeError(msg) => format!("Type error: {}", msg),
            YafError::UndefinedVariable(var) => format!("Undefined variable: {}", var),
            YafError::UndefinedFunction(func) => format!("Undefined function: {}", func),
            YafError::ArgumentMismatch(msg) => format!("Argument mismatch: {}", msg),
            YafError::IoError(msg) => format!("IO error: {}", msg),
            YafError::RuntimeError(msg) => format!("Runtime error: {}", msg),
            YafError::ValueError(msg) => format!("Value error: {}", msg),
            YafError::IndexError(msg) => format!("Index error: {}", msg),
        };
        
        if let Some(loc) = &location {
            eprintln!("{}: {}", "error".bright_red(), message);
            eprintln!("  --> {}:{}:{}", loc.file, loc.line, loc.column);
            
            if let Some(source) = self.source_files.get(&loc.file) {
                self.print_source_context(source, loc);
            }
        } else {
            eprintln!("{}: {}", "error".bright_red(), message);
        }
        
        eprintln!();
    }
    
    fn print_source_context(&self, source: &str, location: &SourceLocation) {
        let lines: Vec<&str> = source.lines().collect();
        
        if location.line > 0 && location.line <= lines.len() {
            let line_num = location.line;
            let source_line = lines[line_num - 1];
            
            // Mostrar líneas de contexto (antes y después del error)
            let context_start = if line_num > 2 { line_num - 2 } else { 1 };
            let context_end = (line_num + 1).min(lines.len());
            
            // Línea vacía para separar
            eprintln!("     |");
            
            // Mostrar contexto antes del error
            for i in context_start..line_num {
                if i <= lines.len() {
                    let line_str = format!("{:4} |", i);
                    eprintln!("{} {}", line_str.bright_blue(), lines[i - 1].dimmed());
                }
            }
            
            // Mostrar la línea con el error
            let line_num_str = format!("{:4} |", line_num);
            eprintln!("{} {}", line_num_str.bright_blue().bold(), source_line);
            
            // Mostrar el indicador del error
            let prefix_spaces = " ".repeat(line_num_str.len() + 1);
            let error_column = location.column.saturating_sub(1); // Ajustar para índice base 0
            let column_spaces = " ".repeat(error_column);
            let carets = if location.length > 1 {
                "^".repeat(location.length)
            } else {
                "^".to_string()
            };
            
            eprintln!("{}{}{}", prefix_spaces, column_spaces, carets.bright_red().bold());
            
            // Mostrar texto específico del error si existe
            if let Some(error_text) = &location.source_text {
                eprintln!("{}{}{} {} {}", 
                    prefix_spaces, column_spaces, 
                    "|".bright_red().bold(), 
                    "error text:".bright_red(), 
                    error_text.bright_yellow()
                );
            }
            
            // Mostrar contexto después del error
            for i in (line_num + 1)..=context_end {
                if i <= lines.len() {
                    let line_str = format!("{:4} |", i);
                    eprintln!("{} {}", line_str.bright_blue(), lines[i - 1].dimmed());
                }
            }
            
            eprintln!("     |");
        }
    }
}
