// Core modules
mod core;
mod backend;
mod runtime;
mod libs;
mod error;
mod cli;
mod diagnostics;

use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use error::YafError;
use clap::Parser as ClapParser;
use colored::*;
use tracing::info;

// Use the new modular structure
use crate::core::{Lexer, Parser, Program, TypeChecker};
// Backend types imported as needed
#[cfg(feature = "llvm-backend")]
use crate::backend::llvm::LLVMCodeGenerator;
use crate::backend::c::CodeGenerator;
use crate::cli::{Args, Commands};
use crate::diagnostics::DiagnosticEngine;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    match args.command {
        Commands::Compile { 
            input, 
            output, 
            backend, 
            emit_ir, 
            emit_llvm, 
            emit_asm, 
            keep_temps, 
            lto, 
            debug 
        } => {
            compile_program(&input, output, backend, emit_ir, emit_llvm, emit_asm, keep_temps, lto, debug, args.optimization, &args.target, args.verbose)
        },
        Commands::Run { input, args: run_args, backend } => {
            run_program(&input, run_args, backend, args.optimization, &args.target, args.verbose)
        },
        Commands::Check { input } => {
            check_program(&input, args.verbose)
        },
        Commands::Format { files, write } => {
            format_files(files, write)
        },
        Commands::Lsp => {
            start_lsp()
        },
        Commands::Repl { backend } => {
            start_repl(backend)
        },
        Commands::Info => {
            show_info()
        },
    }
}

fn compile_program(
    input: &Path, 
    output: Option<PathBuf>, 
    backend: cli::Backend,
    emit_ir: bool,
    emit_llvm: bool, 
    emit_asm: bool,
    keep_temps: bool,
    _lto: bool,
    _debug: bool,
    opt_level: u8,
    _target: &str,
    verbose: bool
) -> Result<()> {
    info!("Compiling {} with {} backend", input.display(), backend);
    
    let source = std::fs::read_to_string(input)
        .map_err(|e| anyhow!("Failed to read input file: {}", e))?;
    
    let mut diagnostics = DiagnosticEngine::new();
    diagnostics.add_source(input.to_string_lossy().as_ref(), source.clone());
    
    // Tokenization
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            // Usar la línea y columna exactas del error de lexer
            let (line, column) = match &err {
                YafError::LexErrorWithPosition { line, column, .. } => (*line, *column),
                _ => (1, 1)
            };
            
            diagnostics.from_yaf_error(&err, input.to_string_lossy().as_ref(), line, column);
            diagnostics.emit_all();
            return Err(anyhow!("Lexical analysis failed"));
        }
    };
    
    if verbose {
        info!("Tokenization completed: {} tokens", tokens.len());
    }
    
    // Parsing
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            // Usar la línea y columna exactas del error
            let (line, column) = match &err {
                YafError::ParseErrorWithPosition { line, column, .. } => (*line, *column),
                YafError::LexErrorWithPosition { line, column, .. } => (*line, *column),
                _ => (1, 1)
            };
            
            diagnostics.from_yaf_error(&err, input.to_string_lossy().as_ref(), line, column);
            diagnostics.emit_all();
            return Err(anyhow!("Parsing failed"));
        }
    };
    
    if verbose {
        info!("Parsing completed");
    }
    
    // Type checking
    let mut type_checker = TypeChecker::new();
    if let Err(err) = type_checker.check(&ast) {
        diagnostics.from_yaf_error(&err, input.to_string_lossy().as_ref(), 1, 1);
        diagnostics.emit_all();
        return Err(anyhow!("Type checking failed"));
    }
    
    if verbose {
        info!("Type checking completed");
    }
    
    let output_name = output.unwrap_or_else(|| {
        input.with_extension("")
    });
    
    match backend {
        cli::Backend::Llvm => {
            #[cfg(feature = "llvm-backend")]
            {
                compile_with_llvm(&ast, &output_name, emit_ir, emit_llvm, emit_asm, opt_level, _target, verbose)
            }
            #[cfg(not(feature = "llvm-backend"))]
            {
                println!("{} LLVM backend not available. Use --features llvm-backend to enable.", "⚠".yellow());
                compile_with_c(&ast, &output_name, keep_temps, opt_level, verbose)
            }
        },
        cli::Backend::Cranelift => {
            compile_with_cranelift(&ast, &output_name, opt_level, verbose)
        },
        cli::Backend::C => {
            compile_with_c(&ast, &output_name, keep_temps, opt_level, verbose)
        },
    }
}

#[cfg(feature = "llvm-backend")]
fn compile_with_llvm(
    ast: &Program, 
    output: &Path, 
    emit_ir: bool, 
    emit_llvm: bool, 
    emit_asm: bool,
    opt_level: u8,
    _target: &str,
    verbose: bool
) -> Result<()> {
    use inkwell::context::Context;
    use inkwell::OptimizationLevel;
    
    let context = Context::create();
    let opt_level = match opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => OptimizationLevel::Default,
    };
    
    let mut codegen = LLVMCodeGenerator::new(&context, "yaf_program", opt_level);
    
    if verbose {
        info!("Generating LLVM IR...");
    }
    
    codegen.generate(ast.clone())?;
    codegen.verify()?;
    
    if emit_llvm {
        let llvm_ir = codegen.emit_llvm_ir();
        let llvm_file = output.with_extension("ll");
        std::fs::write(&llvm_file, llvm_ir)?;
        println!("{} LLVM IR written to: {}", "✓".green(), llvm_file.display());
    }
    
    if !emit_ir && !emit_asm {
        // Generate object file
        let obj_file = output.with_extension("o");
        codegen.emit_to_file(&obj_file)?;
        
        if verbose {
            info!("Object file generated: {}", obj_file.display());
        }
        
        // Link to create executable
        link_executable(&obj_file, output)?;
        
        // Clean up object file if not keeping temps
        std::fs::remove_file(&obj_file).ok();
        
        println!("{} Executable created: {}", "✓".green(), output.display());
    }
    
    Ok(())
}

fn compile_with_cranelift(ast: &Program, output: &Path, opt_level: u8, verbose: bool) -> Result<()> {
    // TODO: Implement Cranelift backend
    println!("{} Cranelift backend not yet implemented", "⚠".yellow());
    compile_with_c(ast, output, false, opt_level, verbose)
}

fn compile_with_c(ast: &Program, output: &Path, keep_temps: bool, opt_level: u8, verbose: bool) -> Result<()> {
    if verbose {
        info!("Generating C code...");
    }
    
    let mut codegen = CodeGenerator::new();
    let c_code = codegen.generate(ast.clone())?;
    
    let c_file = output.with_extension("c");
    std::fs::write(&c_file, c_code)?;
    
    if verbose {
        info!("C code generated: {}", c_file.display());
    }
    
    // Compile C code with YAF runtime
    let runtime_file = Path::new("runtime/yaf_runtime.c");
    let mut compile_cmd = std::process::Command::new("clang");
    compile_cmd
        .arg(&c_file)
        .arg(runtime_file)
        .arg("-o")
        .arg(output)
        .arg("-lm");
    
    match opt_level {
        0 => { compile_cmd.arg("-O0"); },
        1 => { compile_cmd.arg("-O1"); },
        2 => { compile_cmd.arg("-O2"); },
        3 => { compile_cmd.arg("-O3"); },
        _ => { compile_cmd.arg("-O2"); },
    }
    
    let output_result = compile_cmd.output()?;
    
    if !output_result.status.success() {
        eprintln!("{} C compilation failed:", "✗".red());
        eprintln!("{}", String::from_utf8_lossy(&output_result.stderr));
        return Err(anyhow!("C compilation failed"));
    }
    
    if !keep_temps {
        std::fs::remove_file(&c_file).ok();
    }
    
    println!("{} Executable created: {}", "✓".green(), output.display());
    Ok(())
}

fn link_executable(obj_file: &Path, output: &Path) -> Result<()> {
    // First, compile the YAF runtime
    let yaf_runtime_path = Path::new("runtime/yaf_runtime.c");
    let yaf_obj_path = output.with_extension("yaf_runtime.o");
    
    if yaf_runtime_path.exists() {
        let mut compile_runtime_cmd = std::process::Command::new("clang");
        compile_runtime_cmd
            .arg("-c")
            .arg(yaf_runtime_path)
            .arg("-o")
            .arg(&yaf_obj_path)
            .arg("-O2")
            .arg("-I")
            .arg("runtime"); // Include runtime headers
        
        let runtime_output = compile_runtime_cmd.output()?;
        if !runtime_output.status.success() {
            eprintln!("{} YAF runtime compilation failed:", "⚠".yellow());
            eprintln!("{}", String::from_utf8_lossy(&runtime_output.stderr));
            return Err(anyhow!("YAF runtime compilation failed"));
        }
    }
    
    // Then, compile the GC runtime
    let gc_runtime_path = Path::new("src/gc_runtime.c");
    let gc_obj_path = output.with_extension("gc.o");
    
    if gc_runtime_path.exists() {
        let mut compile_gc_cmd = std::process::Command::new("clang");
        compile_gc_cmd
            .arg("-c")
            .arg(gc_runtime_path)
            .arg("-o")
            .arg(&gc_obj_path)
            .arg("-O2");
        
        let gc_output = compile_gc_cmd.output()?;
        if !gc_output.status.success() {
            eprintln!("{} GC runtime compilation failed:", "⚠".yellow());
            eprintln!("{}", String::from_utf8_lossy(&gc_output.stderr));
            // Continue without GC runtime
        }
    }
    
    let mut link_cmd = std::process::Command::new("clang");
    link_cmd
        .arg(obj_file)
        .arg("-o")
        .arg(output)
        .arg("-lm");
    
    // Add YAF runtime (required)
    if yaf_obj_path.exists() {
        link_cmd.arg(&yaf_obj_path);
    }
    
    // Add GC runtime if it was compiled successfully
    if gc_obj_path.exists() {
        link_cmd.arg(&gc_obj_path);
    }
    
    let output_result = link_cmd.output()?;
    
    if !output_result.status.success() {
        eprintln!("{} Linking failed:", "✗".red());
        eprintln!("{}", String::from_utf8_lossy(&output_result.stderr));
        return Err(anyhow!("Linking failed"));
    }
    
    Ok(())
}

fn run_program(input: &Path, args: Vec<String>, backend: cli::Backend, opt_level: u8, target: &str, verbose: bool) -> Result<()> {
    let temp_output = input.with_extension("");
    
    compile_program(
        input, 
        Some(temp_output.clone()), 
        backend, 
        false, false, false, false, false, false,
        opt_level, target, verbose
    )?;
    
    if verbose {
        info!("Running program: {}", temp_output.display());
    }
    
    let mut run_cmd = std::process::Command::new(&temp_output);
    run_cmd.args(&args);
    
    let status = run_cmd.status()?;
    
    // Clean up temporary executable
    std::fs::remove_file(&temp_output).ok();
    
    if !status.success() {
        return Err(anyhow!("Program execution failed"));
    }
    
    Ok(())
}

fn check_program(input: &Path, verbose: bool) -> Result<()> {
    let source = std::fs::read_to_string(input)?;
    
    let mut diagnostics = DiagnosticEngine::new();
    diagnostics.add_source(input.to_string_lossy().as_ref(), source.clone());
    
    let mut has_errors = false;
    
    // Tokenization
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            let (line, column) = match &err {
                YafError::LexErrorWithPosition { line, column, .. } => (*line, *column),
                _ => (1, 1)
            };
            diagnostics.from_yaf_error(&err, input.to_string_lossy().as_ref(), line, column);
            has_errors = true;
            vec![] // Return empty tokens to continue checking
        }
    };
    
    if verbose && !has_errors {
        info!("Tokenization completed: {} tokens", tokens.len());
    }
    
    // Parsing (only if tokenization succeeded)
    let ast = if !has_errors {
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(ast) => Some(ast),
            Err(err) => {
                let (line, column) = match &err {
                    YafError::ParseErrorWithPosition { line, column, .. } => (*line, *column),
                    YafError::LexErrorWithPosition { line, column, .. } => (*line, *column),
                    _ => (1, 1)
                };
                diagnostics.from_yaf_error(&err, input.to_string_lossy().as_ref(), line, column);
                has_errors = true;
                None
            }
        }
    } else {
        None
    };
    
    if verbose && ast.is_some() {
        info!("Parsing completed");
    }
    
    // Type checking (only if parsing succeeded)
    if let Some(ast) = ast {
        let mut type_checker = TypeChecker::new();
        if let Err(err) = type_checker.check(&ast) {
            diagnostics.from_yaf_error(&err, input.to_string_lossy().as_ref(), 1, 1);
            has_errors = true;
        } else if verbose {
            info!("Type checking completed");
        }
    }
    
    // Emit diagnostics and show result
    diagnostics.emit_all();
    
    if has_errors {
        println!("{} Syntax or type errors found", "✗".red());
        Err(anyhow!("Errors found"))
    } else {
        println!("{} Syntax and type check passed", "✓".green());
        Ok(())
    }
}

fn format_files(_files: Vec<PathBuf>, _write: bool) -> Result<()> {
    println!("{} Formatter not yet implemented", "⚠".yellow());
    Ok(())
}

fn start_lsp() -> Result<()> {
    println!("{} LSP server not yet implemented", "⚠".yellow());
    Ok(())
}

fn start_repl(_backend: cli::Backend) -> Result<()> {
    println!("{} REPL not yet implemented", "⚠".yellow());
    Ok(())
}

fn show_info() -> Result<()> {
    println!("{}", "Yaf Programming Language".bright_cyan().bold());
    println!("Version: 0.1.0");
    println!("Built with: {}", "Rust + LLVM".bright_green());
    println!();
    println!("Features:");
    println!("  {} LLVM backend for optimal performance", "✓".green());
    println!("  {} Modern error diagnostics", "✓".green());
    println!("  {} Multiple backends (LLVM, C)", "✓".green());
    println!("  {} Static typing with inference", "✓".green());
    println!("  {} Memory safety", "✓".green());
    println!();
    println!("Backends:");
    println!("  {} LLVM - High performance, best optimization", "🚀".bright_blue());
    println!("  {} C - Maximum compatibility", "🔧".bright_yellow());
    println!("  {} Cranelift - Fast compilation (coming soon)", "⚡".bright_magenta());
    
    Ok(())
}