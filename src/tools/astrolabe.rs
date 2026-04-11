//! The Astrolabe Tool ("Astrolabe")
//!
//! This module implements the "Astrolabe" functionality, which traces
//! the execution of a ΓΛΩΣΣΑ program step-by-step and displays
//! the current state of variables in the interpreter's environment.
//!
//! # Purpose
//!
//! "Astrolabe" enables users to visualize the actual execution flow
//! and state transitions of their code directly, similar to a stepping debugger.

use crate::tools::interpreter::Interpreter;
use crate::tools::runner::{analyze_source, load_source};
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use miette::Result;
use std::path::Path;

/// Run the Astrolabe tool on a file
pub fn run_astrolabe(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀστρολάβος (Visualizing State)", "🧭");

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
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    status.success();
    println!();

    let mut interpreter = Interpreter::new();
    let mut step = 1;

    for stmt in &program.statements {
        if let Err(e) = interpreter.eval_statement(stmt) {
            println!("Execution error at step {}: {:?}", step, e);
            break;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Variable").add_attribute(Attribute::Bold),
                Cell::new("Value").add_attribute(Attribute::Bold),
            ]);

        // Access the global scope directly
        if let Some(scope) = interpreter.env.last() {
            let mut keys: Vec<&String> = scope.keys().collect();
            keys.sort(); // Sort keys for consistent output
            for key in keys {
                table.add_row(vec![
                    Cell::new(key),
                    Cell::new(scope.get(key).unwrap().to_string()),
                ]);
            }
        }

        println!("Step {}:", step);
        println!("{table}");
        println!();
        step += 1;
    }

    // Print final output if any
    let final_out = interpreter.get_output();
    if !final_out.is_empty() {
        println!("Output:");
        println!("{}", final_out);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_run_astrolabe_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_astrolabe(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_astrolabe_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("astrolabe_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ 10 ἔστω.\n".as_bytes())
                .unwrap();
        }

        // Running it should succeed
        let result = run_astrolabe(&input_path);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }
}
