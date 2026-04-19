//! The Scholar (ὁ Σχολαστικός) - API Doc Generator
//!
//! This module implements the "Scholar" tool, which generates Markdown documentation
//! for a ΓΛΩΣΣΑ program's APIs, including its defined structs, traits, and functions.

use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Runs the Scholar tool to generate Markdown documentation from Glossa code.
pub fn run_scholar(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Συγγραφή (Generating Docs)", "📜");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let mut md = String::new();
    let filename = input.file_name().unwrap_or_default().to_string_lossy();

    md.push_str(&format!("# API Documentation: `{}`\n\n", filename));

    // Document Types (Structs)
    let types: Vec<_> = program.scope.types().collect();
    if !types.is_empty() {
        md.push_str("## Types (Εἴδη)\n\n");
        for (name, type_def) in types {
            md.push_str(&format!("### `{}`\n\n", name));
            if let crate::semantic::GlossaType::Struct { fields, .. } = type_def {
                if !fields.is_empty() {
                    md.push_str("| Field | Type |\n");
                    md.push_str("|-------|------|\n");
                    for (field_name, field_type) in fields {
                        md.push_str(&format!("| `{}` | `{}` |\n", field_name, field_type));
                    }
                    md.push('\n');
                } else {
                    md.push_str("*No fields defined.*\n\n");
                }
            }
        }
    }

    // Document Traits (Characters)
    let traits: Vec<_> = program.scope.traits().collect();
    if !traits.is_empty() {
        md.push_str("## Traits (Χαρακτῆρες)\n\n");
        for (name, trait_def) in traits {
            md.push_str(&format!("### `{}`\n\n", name));
            if !trait_def.methods.is_empty() {
                for method in &trait_def.methods {
                    md.push_str(&format!("* `{}`\n", method.name));
                }
                md.push('\n');
            } else {
                md.push_str("*No methods defined.*\n\n");
            }
        }
    }

    // Document Functions (Verbs)
    let functions: Vec<_> = program.scope.functions().collect();
    if !functions.is_empty() {
        md.push_str("## Functions (Ἔργα)\n\n");
        for func in functions {
            // Reconstruct the signature
            let params = func
                .param_types
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = func
                .return_type
                .as_ref()
                .map_or("Οὐδέν".to_string(), |t| format!("{}", t));
            md.push_str(&format!("### `{}({}) -> {}`\n\n", func.name, params, ret));
        }
    }

    let output_path = input.with_extension("doc.md");
    if let Err(e) = std::fs::write(&output_path, &md) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write documentation file: {}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S C H O L A R".bold().cyan());
    println!("   {}", "API Documentation Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_scholar_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");
        fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

        let result = run_scholar(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("doc.md");
        assert!(output_path.exists());
    }
}
