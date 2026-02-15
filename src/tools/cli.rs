//! The CLI (Command Line Interface) 🏛️
//!
//! This module defines the arguments and subcommands for the `glossa` binary.
//! It uses [`clap`] to parse command line arguments.
//!
//! # Structure
//!
//! The CLI is designed to be intuitive:
//!
//! *   `glossa file.γλ`: Runs the file (default action).
//! *   `glossa run file.γλ`: Explicitly runs the file.
//! *   `glossa build file.γλ`: Compiles to Rust without running.
//! *   `glossa check file.γλ`: Checks for errors without compiling.
//! *   `glossa repl`: Starts the interactive shell.
//! *   `glossa bard file.γλ`: Tells the tale of the code.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The main entry point for command line arguments
#[derive(Parser)]
#[command(name = "glossa")]
#[command(about = "ΓΛΩΣΣΑ - Ancient Greek morphology as programming semantics")]
#[command(version)]
pub struct Cli {
    /// The subcommand to run (run, build, check, etc.)
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run a .γλ file directly (default action if no subcommand provided)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// Available subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Run a .γλ file (compile and execute)
    Run {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Compile a .γλ file to Rust source (but do not run)
    Build {
        /// Input file (.γλ)
        input: PathBuf,

        /// Output file (.rs)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check a .γλ file for syntax and semantic errors
    Check {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Highlight a .γλ file with semantic colors (ANSI output)
    Highlight {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Start the interactive REPL (Read-Eval-Print Loop)
    Repl,

    /// Translate a .γλ file to English logic (Experimental)
    Bard {
        /// Input file (.γλ)
        input: PathBuf,
    },
}
