//! The Tracer (ὁ Ἰχνηλάτης)
//!
//! This module implements an experimental execution tracer that steps through
//! a parsed ΓΛΩΣΣΑ program statement by statement, printing the internal
//! state of the environment (`env`) after each step.
//!
//! # Purpose
//!
//! This provides transparency into the runtime execution, making it easier
//! for developers and learners to see exactly how variables are mutated
//! and when logic branches execute.

use crate::tools::interpreter::Interpreter;
// use crate::tools::narrator::tell_statement; // narrator currently only outputs table rows via `add_statement`
use crate::tools::runner::load_source;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Run the Tracer tool on a file
pub fn run_tracer(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let source = load_source(input)?;
    let program = crate::tools::runner::analyze_source(&source)?;

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T R A C E R".bold().cyan());
    println!("   {}", "Execution State Tracker".italic().dim());
    println!();

    let mut interpreter = Interpreter::new();

    for (i, stmt) in program.statements.iter().enumerate() {
        let step_num = i + 1;
        println!("{}", format!("➤ Step {}", step_num).bold().yellow());

        // Let's just print the raw Statement debug derived from standard Debug
        // to avoid refactoring narrator.rs which outputs directly to tables.
        println!("  {}", format!("{:?}", stmt).italic().dim());

        // Evaluate the statement
        if let Err(e) = interpreter.eval_statement(stmt) {
            println!("  {} {}", "Error:".red().bold(), e);
            return Err(e).into_diagnostic();
        }

        // Print output if any (naive approach: just show what the interpreter has collected)
        // Since we evaluate step by step, we check the length of output before and after
        // To be precise we could track previous length, but for now we just show all scope vars

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Variable")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]);

        if let Some(scope) = interpreter.env.last() {
            if scope.is_empty() {
                table.add_row(vec![
                    Cell::new("(empty)"),
                    Cell::new(""),
                ]);
            } else {
                let mut vars: Vec<_> = scope.iter().collect();
                vars.sort_by_key(|k| k.0);
                for (name, value) in vars {
                    table.add_row(vec![
                        Cell::new(name),
                        Cell::new(value.to_string()),
                    ]);
                }
            }
        }

        println!("{table}");
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_tracer_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("trace_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ 5 ἔστω. ξ λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_tracer(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tracer_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_tracer(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_tracer_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ψ πέντε γίγνεται.\n".as_bytes()).unwrap(); // Assigning to undefined var
        }

        let result = run_tracer(&input_path);
        assert!(result.is_err());
    }
}
