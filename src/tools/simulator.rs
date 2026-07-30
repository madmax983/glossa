//! The Simulator (ὁ Μιμητής)
//!
//! This module implements an experimental interpreter simulator that runs ΓΛΩΣΣΑ programs
//! directly without invoking `rustc`.
//!
//! # Purpose
//!
//! Currently, `glossa run` compiles the program to Rust and then executes the binary.
//! The Simulator skips the Rust compilation step entirely by leveraging the existing
//! `Interpreter` from the `repl` module to evaluate the AST dynamically. This provides
//! instant execution and debugging without the overhead of `rustc`.

use crate::tools::interpreter::Interpreter;
use crate::tools::runner::{analyze_source, load_source};
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use std::path::Path;

/// Run the Simulator tool on a file
pub fn run_simulator(input: &Path) -> miette::Result<()> {
    let source = load_source(input)?;

    let status = Status::start_with_symbol("Μίμησις (Simulating)", "🚀");

    let program = match analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S I M U L A T O R".bold().cyan());
    println!("   {}", "Direct Interpretation Output".italic().dim());
    println!();

    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.run(&program) {
        println!("   {} {}", "Runtime Error:".bold().red(), e);
        return Err(miette::miette!("Simulation failed: {}", e));
    }

    let output = interpreter.get_output();
    if !output.is_empty() {
        println!("{}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_simulator_basic() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test_sim.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε κόσμε» λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;


    #[test]
    fn test_run_simulator_file_error() {
        let path = Path::new("non_existent_file_simulator.γλ");
        let result = run_simulator(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_simulator_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_simulator_runtime_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("runtime_error.γλ");
        std::fs::write(&input_path, "1 0 μέρος λέγε.").unwrap(); // division by zero

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }
}
