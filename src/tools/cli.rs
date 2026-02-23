//! Command Line Interface (CLI) definition
//!
//! This module defines the structure of the command-line arguments and subcommands
//! using the [`clap`] crate. It serves as the entry point for user interaction
//! with the compiler.
//!
//! # The Interface
//!
//! The CLI supports several distinct workflows:
//!
//! * **Execution**: `run` (default) compiles and executes a program.
//! * **Compilation**: `build` only compiles to a binary.
//! * **Development**: `check` verifies syntax/semantics, `highlight` shows colors.
//! * **Learning**: `lookup` explains words, `bard` tells the story of the code.
//! * **Interactive**: `repl` starts the Read-Eval-Print Loop.
//!
//! # Example Usage
//!
//! ```bash
//! # Run a file
//! glossa run main.gl
//!
//! # Just check for errors
//! glossa check main.gl
//!
//! # Look up a word
//! glossa lookup "λόγον"
//! ```

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[cfg(feature = "nova")]
use crate::tools::weaver::WeaverCommands;

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

    /// Start the interactive tutorial (Requires "nova" feature)
    #[cfg(feature = "nova")]
    Mentor,

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

    /// Run tests defined in a .γλ file
    Test {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Visualize the assembled sentence structure (Requires "nova" feature)
    #[cfg(feature = "nova")]
    Mosaic {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Visualize the program architecture as a map (Requires "nova" feature)
    #[cfg(feature = "nova")]
    Map {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Scaffold new code (Requires "nova" feature)
    #[cfg(feature = "nova")]
    Weaver {
        #[command(subcommand)]
        command: WeaverCommands,
    },
}
