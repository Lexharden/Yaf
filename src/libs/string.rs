//! # String Library
//! 
//! String manipulation functions for YAF Language

use crate::runtime::values::Value;
use crate::error::Result;

/// String manipulation operations
#[allow(dead_code)]
pub struct StringOps;

#[allow(dead_code)]
impl StringOps {
    /// Get string length
    pub fn length(s: &Value) -> Result<Value> {
        match s {
            Value::String(string) => Ok(Value::Int(string.len() as i64)),
            _ => Err(crate::error::YafError::TypeError(
                "length() requires a string".to_string()
            )),
        }
    }
    
    /// Convert to uppercase
    pub fn upper(s: &Value) -> Result<Value> {
        match s {
            Value::String(string) => Ok(Value::String(string.to_uppercase())),
            _ => Err(crate::error::YafError::TypeError(
                "upper() requires a string".to_string()
            )),
        }
    }
    
    /// Convert to lowercase
    pub fn lower(s: &Value) -> Result<Value> {
        match s {
            Value::String(string) => Ok(Value::String(string.to_lowercase())),
            _ => Err(crate::error::YafError::TypeError(
                "lower() requires a string".to_string()
            )),
        }
    }
    
    /// String concatenation
    pub fn concat(a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::String(s1), Value::String(s2)) => {
                Ok(Value::String(format!("{}{}", s1, s2)))
            },
            _ => Err(crate::error::YafError::TypeError(
                "concat() requires two strings".to_string()
            )),
        }
    }
    
    /// String substring
    pub fn substring(s: &Value, start: &Value, length: &Value) -> Result<Value> {
        match (s, start, length) {
            (Value::String(string), Value::Int(start_idx), Value::Int(len)) => {
                if *start_idx < 0 || *len < 0 {
                    return Err(crate::error::YafError::IndexError(
                        "Negative indices not allowed".to_string()
                    ));
                }
                let start_pos = *start_idx as usize;
                let length_val = *len as usize;
                
                if start_pos > string.len() {
                    return Ok(Value::String(String::new()));
                }
                
                let end_pos = std::cmp::min(start_pos + length_val, string.len());
                Ok(Value::String(string[start_pos..end_pos].to_string()))
            },
            _ => Err(crate::error::YafError::TypeError(
                "substring() requires string, start index, and length".to_string()
            )),
        }
    }
}
