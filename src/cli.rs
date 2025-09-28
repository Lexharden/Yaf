use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "yaf")]
#[command(about = "🚀 YAF - Yet Another Functional Language with LLVM Backend")]
#[command(long_about = "YAF is a modern compiled programming language featuring:
• Built-in functions for math, strings, I/O, and time operations
• LLVM-powered optimizations and native code generation  
• Automatic memory management with garbage collection
• Type-safe arrays and function calls
• Professional modular architecture")]
#[command(version = "0.1.0")]
#[command(author = "YAF Development Team")]
#[command(after_help = "EXAMPLES:
  yaf run hello.yaf              # Compile and run a YAF program
  yaf compile input.yaf -o app   # Compile to executable
  yaf check syntax.yaf           # Check syntax only
  yaf info                       # Show compiler information

Visit: https://github.com/Lexharden/Yaf.git")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable detailed output and logging
    #[arg(short, long, global = true, help = "Show detailed compilation steps")]
    pub verbose: bool,
    
    /// Optimization level (0=none, 1=basic, 2=aggressive, 3=maximum)
    #[arg(short = 'O', long, global = true, default_value = "1", value_parser = clap::value_parser!(u8).range(0..=3))]
    pub optimization: u8,
    
    /// Target architecture (native, x86_64, aarch64)
    #[arg(long, global = true, default_value = "native", help = "CPU architecture to target")]
    pub target: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 🔧 Compile a YAF program to native executable
    Compile {
        /// YAF source file to compile
        #[arg(help = "Path to .yaf source file")]
        input: PathBuf,
        
        /// Output executable name
        #[arg(short, long, help = "Name of output executable")]
        output: Option<PathBuf>,
        
        /// Code generation backend
        #[arg(short, long, default_value = "llvm", help = "Choose compilation backend")]
        backend: Backend,
        
        /// Emit intermediate representation
        #[arg(long)]
        emit_ir: bool,
        
        /// Emit LLVM IR
        #[arg(long)]
        emit_llvm: bool,
        
        /// Emit assembly
        #[arg(long)]
        emit_asm: bool,
        
        /// Keep intermediate files
        #[arg(long)]
        keep_temps: bool,
        
        /// Link time optimization
        #[arg(long)]
        lto: bool,
        
        /// Enable debug information
        #[arg(short, long)]
        debug: bool,
    },
    
    /// 🚀 Compile and run a YAF program in one step
    Run {
        /// YAF source file to run
        #[arg(help = "Path to .yaf source file")]
        input: PathBuf,
        
        /// Arguments to pass to your program
        #[arg(last = true, help = "Arguments passed to the YAF program")]
        args: Vec<String>,
        
        /// Code generation backend
        #[arg(short, long, default_value = "llvm", help = "Choose compilation backend")]
        backend: Backend,
    },
    
    /// ✅ Check syntax and types without compiling
    Check {
        /// YAF source file to check
        #[arg(help = "Path to .yaf source file")]
        input: PathBuf,
    },
    
    /// 📝 Format YAF source code (coming soon)
    Format {
        /// Input files
        files: Vec<PathBuf>,
        
        /// Write changes to files
        #[arg(long)]
        write: bool,
    },
    
    /// 🔌 Start Language Server Protocol for IDE integration
    Lsp,
    
    /// 💻 Interactive REPL (Read-Eval-Print-Loop)
    Repl {
        /// Backend for JIT compilation
        #[arg(short, long, default_value = "cranelift")]
        backend: Backend,
    },
    
    /// ℹ️  Show compiler version and feature information
    Info,
}

#[derive(Clone, ValueEnum)]
pub enum Backend {
    /// 🔥 LLVM backend - Best performance and optimization
    Llvm,
    /// ⚡ Cranelift backend - Fast compilation (experimental)
    Cranelift,
    /// 🛠️ C backend - Maximum compatibility and portability
    C,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Llvm => write!(f, "llvm"),
            Backend::Cranelift => write!(f, "cranelift"),
            Backend::C => write!(f, "c"),
        }
    }
}