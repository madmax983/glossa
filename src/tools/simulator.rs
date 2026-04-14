//! The Simulator Tool ("Simulator")
//!
//! This module implements the "Simulator" functionality, acting as a debug mode
//! that auto-plays a Glossa program using the tree-walk interpreter and inspects
//! its final environment state.
//!
//! # Purpose
//!
//! The Simulator allows users to execute their programs without compiling them
//! to Rust, while providing deep insights into the runtime state. It dumps the
//! final state of all variables in the environment to a table, which is invaluable
//! for debugging logic and understanding execution flow.

use crate::tools::interpreter::Interpreter;
use crate::tools::runner::{analyze_source, load_source};
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Simulator tool on a file
///
/// Reads the source file, analyzes it, executes it using the Interpreter,
/// and dumps the final runtime environment state.
pub fn run_simulator(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ὑποκριτής (Simulating Execution)", "🎭");

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
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let mut interpreter = Interpreter::new();

    if let Err(e) = interpreter.run(&program) {
        if let crate::tools::interpreter::EvalError::NotImplemented(msg) = &e {
            // It's expected that the simulator doesn't support the full language yet.
            // We'll log a warning and still dump the state.
            status.success();
            println!();
            println!("   {}", format!("⚠️ Προσοχή (Warning): Simulation halted early. Feature not implemented yet: {}", msg).yellow());
        } else {
            status.error("Σφάλμα ἐκτελέσεως (Runtime Error)");
            return Err(miette::miette!("Runtime error: {}", e));
        }
    } else {
        status.success();
    }

    let output = interpreter.get_output();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S I M U L A T O R".bold().cyan());
    println!("   {}", "Execution Output & State".italic().dim());
    println!();

    if !output.is_empty() {
        println!("   {}", "Εκτύπωσις (Standard Output)".yellow().bold());
        for line in output.lines() {
            println!("   | {}", line);
        }
        println!();
    }

    println!("   {}", "Περιβάλλον (Environment State)".green().bold());

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("Scope Level")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Variable (Μεταβλητή)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new("Value (Τιμή)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Green),
    ]);

    let env = interpreter.env();
    let mut has_vars = false;

    for (level, scope) in env.iter().enumerate() {
        // Collect and sort keys for deterministic output
        let mut keys: Vec<_> = scope.keys().collect();
        keys.sort();

        for key in keys {
            let value = &scope[key];
            table.add_row(vec![
                Cell::new(level.to_string()),
                Cell::new(key),
                Cell::new(value.to_string()),
            ]);
            has_vars = true;
        }
    }

    if has_vars {
        println!("{table}");
    } else {
        println!("   {}", "No variables defined.".dim());
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_run_simulator_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("sim_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ πέντε ἔστω.\n«χαῖρε» λέγε.\n".as_bytes())
                .unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_simulator_analysis_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("sim_analysis_err_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("invalid syntax".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_simulator_file_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("sim_file_err_test.γλ");
        // Create a directory instead of a file so `load_source` fails
        std::fs::create_dir_all(&input_path).unwrap();

        let result = run_simulator(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_simulator_not_implemented() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("sim_not_impl_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            // This contains a type definition, which is not implemented in the simulator
            f.write_all("εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_ok()); // Should succeed with a warning, not fail
    }

    #[test]
    fn test_run_simulator_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_simulator(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_simulator_runtime_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("sim_err_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Division by zero
            f.write_all("1 0 μέρος λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_simulator(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Runtime error:"));
    }
}
