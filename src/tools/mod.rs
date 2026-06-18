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
pub use cache::Cache;
#[cfg(feature = "nova")]
pub mod cartographer;
#[cfg(feature = "nova")]
pub mod catalog;
pub mod cli;
pub mod dictionary;
/// The Haruspex (ὁ Ἱεροσκόπος) tool for visualizing the Semantic AST.
///
/// This experimental tool exports the analyzed program as a Graphviz DOT diagram.
#[cfg(feature = "nova")]
pub mod gnomon;
#[cfg(feature = "nova")]
pub mod haruspex;
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

use miette::Result;
use std::io::BufRead;

/// Centralized resolution of the glossa binary path for spawning subprocesses
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn find_glossa_binary() -> String {
    std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
        if std::path::Path::new("target/debug/glossa").exists() {
            "target/debug/glossa".to_string()
        } else if std::path::Path::new("target/release/glossa").exists() {
            "target/release/glossa".to_string()
        } else if std::path::Path::new("target/llvm-cov-target/debug/glossa").exists() {
            "target/llvm-cov-target/debug/glossa".to_string()
        } else if std::path::Path::new("target/debug/glossa.exe").exists() {
            "target/debug/glossa.exe".to_string()
        } else if std::path::Path::new("target/release/glossa.exe").exists() {
            "target/release/glossa.exe".to_string()
        } else {
            "glossa".to_string()
        }
    })
}

pub(crate) fn read_line_bounded<R: BufRead>(
    reader: &mut R,
    buf: &mut String,
    limit: usize,
) -> Result<usize, std::io::Error> {
    use std::io::Read;
    reader.by_ref().take(limit as u64).read_line(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_line_bounded_normal() {
        let mut cursor = Cursor::new(b"hello\nworld");
        let mut buf = String::new();
        let bytes = read_line_bounded(&mut cursor, &mut buf, 100).unwrap();
        assert_eq!(bytes, 6);
        assert_eq!(buf, "hello\n");
    }

    #[test]
    fn test_read_line_bounded_limit() {
        let mut cursor = Cursor::new(b"hello\nworld");
        let mut buf = String::new();
        // Limit is 3. Only "hel" should be read
        let bytes = read_line_bounded(&mut cursor, &mut buf, 3).unwrap();
        assert_eq!(bytes, 3);
        assert_eq!(buf, "hel");
    }

    #[test]
    fn test_read_line_bounded_eof() {
        let mut cursor = Cursor::new(b"hello");
        let mut buf = String::new();
        let bytes = read_line_bounded(&mut cursor, &mut buf, 100).unwrap();
        assert_eq!(bytes, 5);
        assert_eq!(buf, "hello");
    }

    #[test]
    fn test_warden_exploit_infinite_stream() {
        // Exploit attempt: simulate /dev/zero
        // A cursor that repeats 0 infinitely
        struct DevZero;
        impl std::io::Read for DevZero {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                for b in buf.iter_mut() {
                    *b = 0;
                }
                Ok(buf.len())
            }
        }
        impl std::io::BufRead for DevZero {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                static ZEROS: [u8; 1024] = [0; 1024];
                Ok(&ZEROS)
            }
            fn consume(&mut self, _amt: usize) {}
        }

        let mut zero = DevZero;
        let mut buf = String::new();
        let limit = 10_000;

        let bytes = read_line_bounded(&mut zero, &mut buf, limit).unwrap();
        assert_eq!(bytes, limit);
        assert_eq!(buf.len(), limit);
        // It successfully stops and does not OOM
    }
}
