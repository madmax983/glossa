//! The Simulator Tool ("Simulator")
//!
//! This module implements the "Simulator" functionality, which executes a ΓΛΩΣΣΑ
//! program using the built-in tree-walk interpreter and visualizes the runtime state.
//!
//! # Purpose
//!
//! While the primary compilation path translates ΓΛΩΣΣΑ to Rust, the `Simulator`
//! evaluates the AST directly. This provides a "Debug Mode" for users to trace
//! execution, view internal state, and verify logic without a full compilation cycle.

use crate::tools::interpreter::Interpreter;
use crate::tools::runner::{analyze_source, load_source};
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Simulator tool on a file
///
/// Reads the source file, parses it, analyzes it, and runs it through the Interpreter.
pub fn run_simulator(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Προσομοιωτής (Simulating)", "🕹️ ");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀνάλυσης (Analysis Error)");
            return Err(e);
        }
    };

    let mut interpreter = Interpreter::new();
    match interpreter.run(&program) {
        Ok(_) => {
            status.success();
            println!();
            println!("   {}", "Γ Λ Ω Σ Σ Α   S I M U L A T O R".bold().cyan());
            println!("   {}", "Execution Successful".italic().dim());
            println!();
            let output = interpreter.get_output();
            if !output.is_empty() {
                println!("   {}", "Output:".bold());
                for line in output.lines() {
                    println!("   > {}", line.yellow());
                }
            } else {
                println!("   {}", "No output produced.".italic().dim());
            }
            println!();
            Ok(())
        }
        Err(e) => {
            // Check for NotImplemented partial simulation
            if matches!(e, crate::tools::interpreter::EvalError::NotImplemented(_)) {
                status.success(); // The simulator gracefully handles partial AST support
                println!();
                println!("   {}", "Γ Λ Ω Σ Σ Α   S I M U L A T O R".bold().cyan());
                println!("   {}", "Partial Execution".italic().yellow());
                println!();
                println!("   {}", "Notice:".bold());
                println!(
                    "   {}",
                    "Simulation halted because some language features are not yet supported"
                        .yellow()
                );
                println!("   {}", "by the tree-walk interpreter.".yellow());
                println!();
                let output = interpreter.get_output();
                if !output.is_empty() {
                    println!("   {}", "Output before halt:".bold());
                    for line in output.lines() {
                        println!("   > {}", line.yellow());
                    }
                }
                println!();
                Ok(())
            } else {
                status.error("Σφάλμα ἐκτέλεσης (Runtime Error)");
                Err(miette::miette!("Runtime error: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simulator_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_test.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε κόσμε» λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_simulator(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_simulator_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }
}
