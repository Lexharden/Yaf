//! # Mathematical Operations Library
//!
//! Provides mathematical functions and constants for YAF

use crate::runtime::values::Value;
use crate::error::Result;

/// Mathematical constants
#[allow(dead_code)]
pub struct MathConstants;

#[allow(dead_code)]
impl MathConstants {
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
    pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
}

/// Basic mathematical operations
pub struct MathOps;

#[allow(dead_code)]
impl MathOps {
    /// Absolute value
    pub fn abs(value: &Value) -> Result<Value> {
        match value {
            Value::Int(n) => Ok(Value::Int(n.abs())),
            _ => Err(crate::error::YafError::TypeError(
                "abs() requires an integer".to_string()
            )),
        }
    }
    
    /// Maximum of two values
    pub fn max(a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(*x.max(y))),
            _ => Err(crate::error::YafError::TypeError(
                "max() requires two integers".to_string()
            )),
        }
    }
    
    /// Minimum of two values
    pub fn min(a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(*x.min(y))),
            _ => Err(crate::error::YafError::TypeError(
                "min() requires two integers".to_string()
            )),
        }
    }
    
    /// Power function
    pub fn pow(base: &Value, exponent: &Value) -> Result<Value> {
        match (base, exponent) {
            (Value::Int(b), Value::Int(e)) => {
                if *e >= 0 {
                    Ok(Value::Int(b.pow(*e as u32)))
                } else {
                    Err(crate::error::YafError::TypeError(
                        "Negative exponents not supported for integers".to_string()
                    ))
                }
            },
            _ => Err(crate::error::YafError::TypeError(
                "pow() requires two integers".to_string()
            )),
        }
    }
}
