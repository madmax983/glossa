//! The Sibyl (ἡ Σίβυλλα) - Test Scaffolder
//!
//! This module implements the "Sibyl" tool, which parses a ΓΛΩΣΣΑ program
//! and automatically generates test boilerplate (`δοκιμή`) for every function defined.
//!
//! # Purpose
//!
//! The Sibyl foresees the trials ahead. Writing test boilerplate manually is tedious.
//! By inspecting the semantic AST for `FunctionDefinition` nodes, the Sibyl generates
//! a skeleton test block for each function, complete with commented-out placeholder
//! variable initializations based on the function's parameter types, and a failing
//! assertion to encourage Test-Driven Development (TDD).

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Sibyl tool to generate test boilerplate for a Glossa program.
pub fn run_sibyl(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Πρόβλεψις (Foreseeing Trials)", "👁️");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let mut output = String::new();
    let mut found_functions = 0;

    for stmt in &program.statements {
        if let AnalyzedStatement::FunctionDef { name, params, .. } = stmt {
            found_functions += 1;
            writeln!(output, "δοκιμή «{name} should work».").expect("Failed to write string");

            for (param_name, param_type) in params {
                let default_val = param_type.as_ref().map_or("...", generate_default_value);
                writeln!(output, "    // {param_name} {default_val} ἔστω.")
                    .expect("Failed to write string");
            }

            writeln!(output, "    // {name} (...)").expect("Failed to write string");
            writeln!(output, "    ἀληθές ψεῦδος ἰσοῦται. // TODO: Implement test")
                .expect("Failed to write string");
            writeln!(output, "τέλος.\n").expect("Failed to write string");
        }
    }

    if found_functions == 0 {
        status.success();
        println!("No functions found to scaffold tests for.");
        return Ok(());
    }

    let output_path = input.with_extension("tests.γλ");
    if let Err(e) = std::fs::write(&output_path, &output) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write test file: {}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S I B Y L".bold().cyan());
    println!("   {}", "Test Boilerplate Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

fn generate_default_value(ty: &GlossaType) -> &'static str {
    match ty {
        GlossaType::Number => "0",
        GlossaType::String => "«»",
        GlossaType::Boolean => "ἀληθές",
        GlossaType::List(_) => "[ ]",
        _ => "...", // Placeholder for complex/unknown types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_generate_default_value() {
        assert_eq!(generate_default_value(&GlossaType::Number), "0");
        assert_eq!(generate_default_value(&GlossaType::String), "«»");
        assert_eq!(generate_default_value(&GlossaType::Boolean), "ἀληθές");
        assert_eq!(
            generate_default_value(&GlossaType::List(Box::new(GlossaType::Number))),
            "[ ]"
        );
        assert_eq!(generate_default_value(&GlossaType::Unknown), "...");
    }

    #[test]
    fn test_run_sibyl_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("logic.γλ");
        let source = "
        προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ·
            δός ξ ψ ἄθροισμα.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_sibyl(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("tests.γλ");
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("δοκιμή «προσθεσις should work»."));
        assert!(content.contains("// ξ 0 ἔστω."));
        assert!(content.contains("// ψ 0 ἔστω."));
        assert!(content.contains("ἀληθές ψεῦδος ἰσοῦται. // TODO: Implement test"));
        assert!(content.contains("τέλος."));
    }

    #[test]
    fn test_run_sibyl_no_functions() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("logic.γλ");
        let source = "ξ 10 ἔστω.";
        fs::write(&input_path, source).unwrap();

        let result = run_sibyl(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("tests.γλ");
        assert!(!output_path.exists());
    }
}
