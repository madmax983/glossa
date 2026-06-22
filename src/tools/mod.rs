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
pub(crate) mod alchemist;
/// The Auditor (Λογιστής) tool for static analysis.
///
/// This experimental tool analyzes Glossa code to detect unused variables,
/// unnecessary mutable bindings, and other code quality issues.
#[cfg(feature = "nova")]
pub(crate) mod auditor;
pub(crate) mod cache;
pub use cache::Cache;
#[cfg(feature = "nova")]
pub(crate) mod cartographer;
#[cfg(feature = "nova")]
pub(crate) mod catalog;
pub(crate) mod cli;
pub(crate) mod dictionary;
/// The Haruspex (ὁ Ἱεροσκόπος) tool for visualizing the Semantic AST.
///
/// This experimental tool exports the analyzed program as a Graphviz DOT diagram.
#[cfg(feature = "nova")]
pub(crate) mod gnomon;
#[cfg(feature = "nova")]
pub(crate) mod haruspex;
pub(crate) mod highlight;
#[cfg(feature = "nova")]
pub(crate) mod interpreter;
#[cfg(feature = "nova")]
pub(crate) mod labyrinth;
#[cfg(feature = "nova")]
pub(crate) mod mentor;
#[cfg(feature = "nova")]
pub(crate) mod mosaic;
pub(crate) mod narrator;
/// The Papyrus (Πάπυρος) tool for SQL schema generation.
///
/// This experimental tool reads Glossa type definitions and automatically
/// generates corresponding SQL `CREATE TABLE` statements.
#[cfg(feature = "nova")]
pub(crate) mod papyrus;
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
#[cfg(feature = "nova")]
pub(crate) mod scholar;
pub(crate) mod tester;
pub(crate) mod ui;
#[cfg(feature = "nova")]
pub(crate) mod weave;

// Facade exports
pub use cli::{Cli, Commands};
pub use dictionary::lookup_word;
pub use narrator::tell_tale;
pub use repl::run_repl;
pub use runner::{
    analyze_source, bard_file, build_file, check_file, highlight_file, report_file, run_file,
};
pub use tester::run_tests;

#[cfg(feature = "nova")]
pub use alchemist::{run_alchemist, transpile_to_python};
#[cfg(feature = "nova")]
pub use auditor::run_auditor;
#[cfg(feature = "nova")]
pub use cartographer::run_map;
#[cfg(feature = "nova")]
pub use catalog::run_catalog;
#[cfg(feature = "nova")]
pub use gnomon::run_gnomon;
#[cfg(feature = "nova")]
pub use haruspex::run_haruspex;
pub use highlight::highlight;
#[cfg(feature = "nova")]
pub use interpreter::{EvalError, Interpreter, Value};
#[cfg(feature = "nova")]
pub use labyrinth::{run_labyrinth, run_labyrinth_inner};
#[cfg(feature = "nova")]
pub use mentor::run_mentor;
#[cfg(feature = "nova")]
pub use mosaic::{run_mosaic, run_mosaic_inner};
#[cfg(feature = "nova")]
pub use papyrus::run_papyrus;
#[cfg(feature = "nova")]
pub use scholar::run_scholar;
#[cfg(feature = "nova")]
pub use weave::run_weave;
