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
/// The Auditor (Λογιστής) tool for static analysis.
///
/// This experimental tool analyzes Glossa code to detect unused variables,
/// unnecessary mutable bindings, and other code quality issues.
#[cfg(feature = "nova")]
pub mod auditor;
pub(crate) mod cache;
#[cfg(feature = "nova")]
pub mod theater;
pub use cache::Cache;
#[cfg(feature = "nova")]
pub mod cartographer;
#[cfg(feature = "nova")]
pub mod catalog;
pub mod cli;
pub mod dictionary;
pub mod highlight;
#[cfg(feature = "nova")]
pub mod interpreter;
#[cfg(feature = "nova")]
pub mod labyrinth;
#[cfg(feature = "nova")]
pub mod mentor;
#[cfg(feature = "nova")]
pub mod mosaic;
pub mod narrator;
/// The Papyrus (Πάπυρος) tool for SQL schema generation.
///
/// This experimental tool reads Glossa type definitions and automatically
/// generates corresponding SQL `CREATE TABLE` statements.
#[cfg(feature = "nova")]
pub mod papyrus;
pub mod repl;
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
/// use glossa::tools::runner::run_file;
/// use std::path::Path;
///
/// // Execute a Glossa file directly from its path
/// let input = Path::new("main.γλ");
/// if let Err(e) = run_file(&input) {
///     eprintln!("Execution failed: {}", e);
/// }
/// ```
pub mod runner;
#[cfg(feature = "nova")]
pub mod scholar;
pub mod tester;
pub(crate) mod ui;
#[cfg(feature = "nova")]
pub mod weave;
