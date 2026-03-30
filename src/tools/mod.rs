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
//! * [`report`]: **The Scribe** - Detailed compilation reports and error diagnostics.
//! * [`cache`]: **The Vault** - Incremental compilation cache to speed up builds.
//! * [`dictionary`]: **The Lexicon** - Word lookup utility for the built-in dictionary.
//! * [`narrator`]: **The Bard** - Experimental "code-to-story" translator.
//! * [`tester`]: **The Judge** - Built-in test runner for unit tests defined in ΓΛΩΣΣΑ files.
//! * [`ui`]: **The Stage** - Terminal UI helpers (status spinners, emojis, etc.).

#[cfg(feature = "nova")]
pub(crate) mod alchemist;
pub(crate) mod cache;
#[cfg(feature = "nova")]
pub(crate) mod cartographer;
pub(crate) mod cli;
pub(crate) mod dictionary;
pub(crate) mod highlight;
#[cfg(feature = "nova")]
pub(crate) mod interpreter;
#[cfg(feature = "nova")]
pub(crate) mod mentor;
#[cfg(feature = "nova")]
pub(crate) mod mosaic;
pub(crate) mod narrator;
pub(crate) mod repl;
pub(crate) mod report;
pub(crate) mod runner;
pub(crate) mod tester;
pub(crate) mod ui;
#[cfg(feature = "nova")]
pub(crate) mod weave;

#[cfg(feature = "nova")]
pub use alchemist::{run_alchemist, transpile_to_python};
pub use cache::Cache;
#[cfg(feature = "nova")]
pub use cartographer::{generate_map, run_map};
pub use cli::{Cli, Commands};
pub use dictionary::lookup_word;
#[cfg(feature = "nova")]
pub use interpreter::Interpreter;
#[cfg(feature = "nova")]
pub use mentor::run_mentor;
#[cfg(feature = "nova")]
pub use mosaic::{run_mosaic, run_mosaic_inner};
pub use narrator::tell_tale;
pub use repl::run_repl;
pub use report::{CompilationReport, GlossaReport, ProgramStats};
pub use runner::{bard_file, build_file, check_file, highlight_file, run_file};
pub use tester::run_tests;
pub use ui::Status;
#[cfg(feature = "nova")]
pub use weave::run_weave;
pub use highlight::highlight;
