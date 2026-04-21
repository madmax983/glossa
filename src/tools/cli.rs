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
//! * **Testing**: `test` runs unit tests defined in the source file.
//! * **Compilation**: `build` only compiles to a binary.
//! * **Development**: `check` verifies syntax/semantics, `highlight` shows colors.
//! * **Learning**: `lookup` explains words, `bard` tells the story of the code.
//! * **Interactive**: `repl` starts the Read-Eval-Print Loop.
//!
//! ## Experimental Tools (Nova Feature)
//!
//! Some advanced visualization tools require the `nova` feature flag to be enabled
//! during compilation of the compiler itself.
//!
//! * **Mentor**: `mentor` starts an interactive tutorial.
//! * **Mosaic**: `mosaic` visualizes the sentence structure assembly.
//! * **Map**: `map` generates architecture diagrams.
//!
//! # Example Usage
//!
//! ```bash
//! # Run a file
//! glossa run main.gl
//!
//! # Run tests
//! glossa test main.gl
//!
//! # Just check for errors
//! glossa check main.gl
//!
//! # Look up a word
//! glossa lookup "λόγον"
//!
//! # Visualize sentence structure (requires feature 'nova')
//! glossa mosaic main.gl
//! ```

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The main entry point configuration for the Glossa compiler CLI.
///
/// This struct defines the root of the command line interface, parsing the user's
/// inputs into understandable commands that the compiler can act upon. We use `clap`
/// to automatically generate help text and handle argument parsing.
///
/// # Examples
///
/// ```rust
/// use glossa::tools::cli::Cli;
/// use clap::Parser;
///
/// // You can parse arguments from an iterator, which is useful for testing!
/// let args = Cli::parse_from(&["glossa", "run", "hero.γλ"]);
/// assert!(args.command.is_some());
/// ```
#[derive(Parser)]
#[command(name = "glossa")]
#[command(about = "ΓΛΩΣΣΑ - Ancient Greek morphology as programming semantics")]
#[command(version)]
pub struct Cli {
    /// The specific action the user wishes the compiler to perform (e.g., compile, run, test).
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run a .γλ file directly (default action)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

/// The available subcommands for the Glossa CLI.
///
/// Each variant represents a distinct workflow or tool within the compiler suite.
/// By explicitly defining these as an enum, we ensure that the user's intent
/// is strictly typed and easily matchable in the main execution loop.
///
/// # Examples
///
/// ```rust
/// use glossa::tools::cli::Commands;
/// use std::path::PathBuf;
///
/// let run_cmd = Commands::Run { input: PathBuf::from("main.γλ") };
/// match run_cmd {
///     Commands::Run { input } => assert_eq!(input.to_str().unwrap(), "main.γλ"),
///     _ => panic!("Expected Run command"),
/// }
/// ```
#[derive(Subcommand)]
pub enum Commands {
    /// Run a .γλ file (default)
    Run {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Generate a language metrics dashboard
    Report {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Visualize the control flow graph as a map (Requires "nova" feature)
    Labyrinth {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Start the interactive tutorial (Requires "nova" feature)
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
    Mosaic {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Visualize the program architecture as a map (Requires "nova" feature)
    Map {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Generate a Markdown Rosetta Stone (Requires "nova" feature)
    Weave {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Transpile a .γλ file to Python (Requires "nova" feature)
    Alchemist {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Transpile a .γλ file to SQL CREATE TABLE schema (Requires "nova" feature)
    Papyrus {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Run the Auditor to find code smells (Requires "nova" feature)
    Audit {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Explore the built-in lexicon (Requires "nova" feature)
    Catalog,

    /// Generate an API documentation Markdown file (Requires "nova" feature)
    Scholar {
        /// Input file (.γλ)
        input: PathBuf,
    },
}

impl Commands {
    /// Extracts the input file path for commands that operate on a file.
    pub fn input_path(&self) -> Option<&PathBuf> {
        match self {
            Commands::Run { input } => Some(input),
            Commands::Report { input } => Some(input),
            Commands::Labyrinth { input } => Some(input),
            Commands::Mentor => None,
            Commands::Build { input, .. } => Some(input),
            Commands::Check { input } => Some(input),
            Commands::Highlight { input } => Some(input),
            Commands::Repl => None,
            Commands::Bard { input } => Some(input),
            Commands::Lookup { .. } => None,
            Commands::Test { input } => Some(input),
            Commands::Mosaic { input } => Some(input),
            Commands::Map { input } => Some(input),
            Commands::Weave { input } => Some(input),
            Commands::Alchemist { input } => Some(input),
            Commands::Papyrus { input } => Some(input),
            Commands::Audit { input } => Some(input),
            Commands::Catalog => None,
            Commands::Scholar { input } => Some(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands_input_path() {
        let p = PathBuf::from("test.γλ");

        let commands_with_input = vec![
            Commands::Run { input: p.clone() },
            Commands::Report { input: p.clone() },
            Commands::Labyrinth { input: p.clone() },
            Commands::Build {
                input: p.clone(),
                output: None,
            },
            Commands::Check { input: p.clone() },
            Commands::Highlight { input: p.clone() },
            Commands::Bard { input: p.clone() },
            Commands::Test { input: p.clone() },
            Commands::Mosaic { input: p.clone() },
            Commands::Map { input: p.clone() },
            Commands::Weave { input: p.clone() },
            Commands::Alchemist { input: p.clone() },
            Commands::Papyrus { input: p.clone() },
            Commands::Audit { input: p.clone() },
            Commands::Scholar { input: p.clone() },
        ];

        for cmd in commands_with_input {
            assert_eq!(cmd.input_path(), Some(&p));
        }

        let commands_without_input = vec![
            Commands::Mentor,
            Commands::Repl,
            Commands::Lookup {
                word: "test".to_string(),
            },
            Commands::Catalog,
        ];

        for cmd in commands_without_input {
            assert_eq!(cmd.input_path(), None);
        }
    }
}
