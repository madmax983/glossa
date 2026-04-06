//! The Simulator Tool ("Simulator")
//!
//! This module implements the "Simulator" functionality, which allows executing
//! ΓΛΩΣΣΑ programs directly via the internal tree-walk interpreter without
//! compiling to Rust or invoking `rustc`.
//!
//! # Purpose
//!
//! Compiling to Rust can be slow due to `rustc` invocation overhead. The Simulator
//! allows users to rapidly prototype and execute scripts by interpreting the semantic
//! AST directly in memory. It provides immediate feedback and an alternative execution
//! environment for constrained systems.

use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::interpreter::Interpreter;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Run a program directly via the Interpreter
pub fn run_simulator(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let source = load_source(input)?;
    let status = Status::start_with_symbol("Προσομοίωσις (Simulating)", "🎭");

    // 1. Parse
    let ast = match parse(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα ἀναγνώσεως (Parse Error)");
            return Err(e.into());
        }
    };

    // 2. Analyze
    let analyzed = match analyze_program(&ast) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα σημασιολογίας (Semantic Error)");
            return Err(e.into());
        }
    };

    status.success();
    println!("🎭 Προσομοίωσις ἑτοίμη (Simulation Ready)");
    println!();

    // 3. Interpret
    let mut interpreter = Interpreter::new();
    interpreter.run(&analyzed).into_diagnostic()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_simulator_basic_execution() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "«χαῖρε» λέγε.").unwrap();

        let result = run_simulator(file.path());
        assert!(
            result.is_ok(),
            "Simulator failed to execute basic print statement"
        );
    }

    #[test]
    fn test_simulator_missing_file() {
        let result = run_simulator(Path::new("does_not_exist.γλ"));
        assert!(result.is_err(), "Simulator should fail on missing files");
    }
}
