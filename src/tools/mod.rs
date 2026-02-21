pub mod dictionary;
pub mod highlight;
pub mod narrator;
pub mod repl;
pub mod runner;
pub mod tester;
pub mod ui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "glossa")]
#[command(about = "ΓΛΩΣΣΑ - Ancient Greek morphology as programming semantics")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run a .γλ file directly (default action)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a .γλ file (default)
    Run {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Compile a .γλ file to Rust source
    Build {
        /// Input file (.γλ)
        input: PathBuf,

        /// Output file (.rs)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check a .γλ file without running
    Check {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Highlight a .γλ file with semantic colors
    Highlight {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Start the interactive REPL
    Repl,

    /// Translate a .γλ file to English logic (Experimental)
    Bard {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Lookup a word in the built-in lexicon
    Lookup {
        /// The Greek word to analyze
        word: String,
    },

    /// Run tests defined in a .γλ file (Requires "nova" feature)
    #[cfg(feature = "nova")]
    Test {
        /// Input file (.γλ)
        input: PathBuf,
    },
}
