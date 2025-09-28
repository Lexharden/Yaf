//! # I/O Operations Library
//!
//! File operations and console I/O for YAF

use std::fs;
use std::io::{self, Write};

use crate::runtime::values::Value;
use crate::error::Result;

/// File operations
#[allow(dead_code)]
pub struct FileOps;

#[allow(dead_code)]
impl FileOps {
    /// Read file contents
    pub fn read_file(path: &Value) -> Result<Value> {
        match path {
            Value::String(file_path) => {
                match fs::read_to_string(file_path) {
                    Ok(contents) => Ok(Value::String(contents)),
                    Err(e) => Err(crate::error::YafError::IoError(
                        format!("Failed to read file '{}': {}", file_path, e)
                    )),
                }
            },
            _ => Err(crate::error::YafError::TypeError(
                "read_file() requires a string path".to_string()
            )),
        }
    }
    
    /// Write file contents
    pub fn write_file(path: &Value, content: &Value) -> Result<Value> {
        match (path, content) {
            (Value::String(file_path), Value::String(file_content)) => {
                match fs::write(file_path, file_content) {
                    Ok(_) => Ok(Value::Bool(true)),
                    Err(e) => Err(crate::error::YafError::IoError(
                        format!("Failed to write file '{}': {}", file_path, e)
                    )),
                }
            },
            _ => Err(crate::error::YafError::TypeError(
                "write_file() requires string path and content".to_string()
            )),
        }
    }
    
    /// Check if file exists
    pub fn file_exists(path: &Value) -> Result<Value> {
        match path {
            Value::String(file_path) => {
                Ok(Value::Bool(std::path::Path::new(file_path).exists()))
            },
            _ => Err(crate::error::YafError::TypeError(
                "file_exists() requires a string path".to_string()
            )),
        }
    }
}

/// Console operations
#[allow(dead_code)]
pub struct Console;

#[allow(dead_code)]
impl Console {
    /// Print to stdout
    pub fn print(value: &Value) -> Result<Value> {
        print!("{}", value);
        io::stdout().flush().map_err(|e| {
            crate::error::YafError::IoError(format!("Failed to flush stdout: {}", e))
        })?;
        Ok(Value::Bool(true))
    }
    
    /// Print line to stdout
    pub fn println(value: &Value) -> Result<Value> {
        println!("{}", value);
        Ok(Value::Bool(true))
    }
    
    /// Read line from stdin (placeholder)
    pub fn read_line() -> Result<Value> {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Remove trailing newline
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                Ok(Value::String(input))
            },
            Err(e) => Err(crate::error::YafError::IoError(
                format!("Failed to read from stdin: {}", e)
            )),
        }
    }
}
