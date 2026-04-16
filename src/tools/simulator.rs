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

    #[test]
    fn test_run_simulator_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.γλ");
        // Valid syntax, but 'ψ' is not defined, causing a semantic analysis error
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ ὡρίσθη"));
    }

    #[test]
    fn test_run_simulator_not_implemented() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_not_impl.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Struct definitions are not yet supported by the Interpreter
            f.write_all("εἶδος Χ ὁρίζειν { χ ἀριθμοῦ. }.\n".as_bytes()).unwrap();
        }

        // It should succeed but report Partial Execution
        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_runtime_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_runtime_error.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Valid parse, valid semantics, but runtime error (divide by zero)
            f.write_all("1 0 μέρος λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Runtime error"));
    }

    #[test]
    fn test_run_simulator_empty_output() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("simulator_empty_output.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Valid parse, valid semantics, no output
            f.write_all("ξ 10 ἔστω.\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_file_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("file_error.γλ");
        std::fs::write(&input_path, "ξ 10 ἔστω.").unwrap();

        // Simulate a file error (e.g., trying to run a directory as a file)
        let dir_path = dir.path().join("a_directory.γλ");
        std::fs::create_dir(&dir_path).unwrap();

        let result = run_simulator(&dir_path);
        assert!(result.is_err());
    }
}
