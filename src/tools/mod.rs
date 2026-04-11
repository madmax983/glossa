//! Compiler tools and utilities
//!
//! This module contains the various sub-tools that power the ΓΛΩΣΣΑ compiler ecosystem.
//! Each submodule handles a specific aspect of the developer experience or compiler pipeline.
//!
//! # The Toolset
//!
//! * [`cli`]: **The Command Center** - Defines the command-line interface using `clap`.
//! * [`runner`]: **The Engine Room** - Orchestrates the full compilation pipeline (load -> analyze -> compile -> run).
//! * [`repl`]: **The Playground** - An interactive Read-Eval-Print Loop for experimentation.
//! * [`highlight`]: **The Painter** - Syntax highlighting for terminal output.
//! * `report`: **The Scribe** - Detailed compilation reports and error diagnostics.
//! * [`Cache`]: **The Vault** - Incremental compilation cache to speed up builds.
//! * [`dictionary`]: **The Lexicon** - Word lookup utility for the built-in dictionary.
//! * [`narrator`]: **The Bard** - Experimental "code-to-story" translator.
//! * [`tester`]: **The Judge** - Built-in test runner for unit tests defined in ΓΛΩΣΣΑ files.
//! * `ui`: **The Stage** - Terminal UI helpers (status spinners, emojis, etc.).

#[cfg(feature = "nova")]
pub mod alchemist;
#[cfg(feature = "nova")]
pub mod auditor;
pub(crate) mod cache;
pub use cache::Cache;
#[cfg(feature = "nova")]
pub mod cartographer;
pub(crate) mod cli;
pub(crate) mod dictionary;
pub mod highlight;
#[cfg(feature = "nova")]
pub mod interpreter;
#[cfg(feature = "nova")]
pub mod labyrinth;
#[cfg(feature = "nova")]
pub mod mentor;
#[cfg(feature = "nova")]
pub mod mosaic;
pub(crate) mod narrator;
#[cfg(feature = "nova")]
pub mod papyrus;
pub(crate) mod repl;
pub(crate) mod report;
/// The engine room for executing and building Glossa programs
///
/// This module orchestrates the full compilation pipeline from source file to executable binary.
/// It bridges the gap between parsing, semantic analysis, code generation, and the final
/// execution via `rustc`.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::run_file;
/// use std::path::Path;
///
/// // Execute a Glossa file directly from its path
/// let input = Path::new("main.γλ");
/// if let Err(e) = run_file(&input) {
///     eprintln!("Execution failed: {}", e);
/// }
/// ```
pub(crate) mod runner;
pub(crate) mod tester;
pub(crate) mod ui;
#[cfg(feature = "nova")]
pub mod weave;

// Export the necessary CLI and binary items for src/main.rs and integration tests
pub use cli::{Cli, Commands};
pub use dictionary::lookup_word;
pub use narrator::tell_tale;
pub use repl::run_repl;
pub use runner::{bard_file, build_file, check_file, highlight_file, report_file, run_file};
pub use tester::run_tests;
