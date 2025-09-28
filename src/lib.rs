//! # YAF Language - Compiler Library
//! 
//! A modern compiled programming language with LLVM backend
//! 
//! ## Architecture
//! 
//! - **Core**: Essential compiler components (lexer, parser, AST)
//! - **Backend**: Code generation (LLVM, C)
//! - **Libraries**: Built-in libraries (math, net, io, etc.)
//! - **Runtime**: Memory management and garbage collection
//! 

pub mod core;
pub mod backend;
pub mod libs;
pub mod runtime;
pub mod diagnostics;
pub mod error;

// Re-export commonly used types
pub use crate::core::ast::*;
pub use crate::core::lexer::*;
pub use crate::core::parser::*;
pub use crate::error::{YafError, Result};
