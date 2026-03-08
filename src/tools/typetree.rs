//! The TypeTree Tool (τὸ Δένδρον)
//!
//! A visualizer for semantic data models in ΓΛΩΣΣΑ.

use miette::Result;
use std::path::Path;

/// Run the TypeTree tool on a file
pub fn run_typetree(input_path: &Path) -> Result<()> {
    let source = crate::tools::runner::load_source(input_path)?;
    let mut buffer = Vec::new();
    run_typetree_inner(&source, &mut buffer)?;

    let output = String::from_utf8(buffer).expect("Invalid UTF-8");
    println!("{}", output);

    Ok(())
}

use crate::parser::parse;
use crate::semantic::analyze_program;

/// Internal logic for testing
use crate::semantic::GlossaType;
use comfy_table::{Cell, CellAlignment, Table, presets::UTF8_FULL};
use crossterm::style::Stylize;
use miette::IntoDiagnostic;
use std::collections::BTreeMap;

pub(crate) fn run_typetree_inner<W: std::io::Write>(source: &str, writer: &mut W) -> Result<()> {
    // 1. Parse and Analyze
    let ast = parse(source)?;
    let program = analyze_program(&ast)?;

    // 2. Extract Data Models (Structs and Traits)
    // We use BTreeMap to sort them alphabetically by name
    let mut structs: BTreeMap<String, Vec<(String, GlossaType)>> = BTreeMap::new();
    let mut traits: BTreeMap<String, Vec<crate::semantic::AnalyzedMethod>> = BTreeMap::new();

    for (name, ty) in program.scope.types() {
        if let GlossaType::Struct { fields, .. } = ty {
            structs.insert(
                name.to_string(),
                fields
                    .iter()
                    .map(|(n, t)| (n.to_string(), t.clone()))
                    .collect(),
            );
        }
    }

    for (name, trait_def) in program.scope.traits() {
        traits.insert(name.to_string(), trait_def.methods.clone());
    }

    // 3. Build Tree Construction
    let mut tree_out = String::new();

    if structs.is_empty() && traits.is_empty() {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.add_row(vec![
            Cell::new("No semantic data models (εἶδος/χαρακτήρ) found in the current scope.")
                .set_alignment(CellAlignment::Center),
        ]);
        tree_out.push_str(&format!("{}\n", table));
    } else {
        if !structs.is_empty() {
            tree_out.push_str(&format!("📦 {}\n", "Εἴδη (Structs)".bold().cyan()));
            for (i, (name, fields)) in structs.iter().enumerate() {
                let is_last_struct = i == structs.len() - 1 && traits.is_empty();
                let prefix = if is_last_struct {
                    "└── "
                } else {
                    "├── "
                };
                let child_prefix = if is_last_struct { "    " } else { "│   " };

                tree_out.push_str(&format!(
                    "{}{} {}\n",
                    prefix.cyan(),
                    "εἶδος".magenta(),
                    name.clone().bold()
                ));

                for (j, (field_name, field_type)) in fields.iter().enumerate() {
                    let is_last_field = j == fields.len() - 1;
                    let field_prefix = if is_last_field {
                        "└── "
                    } else {
                        "├── "
                    };

                    tree_out.push_str(&format!(
                        "{}{} {}: {}\n",
                        child_prefix.cyan(),
                        field_prefix.cyan(),
                        field_name.clone().green(),
                        field_type.to_string().yellow()
                    ));
                }
            }
        }

        if !traits.is_empty() {
            if !structs.is_empty() {
                tree_out.push_str("│\n");
            }
            tree_out.push_str(&format!("📜 {}\n", "Χαρακτῆρες (Traits)".bold().blue()));
            for (i, (name, methods)) in traits.iter().enumerate() {
                let is_last_trait = i == traits.len() - 1;
                let prefix = if is_last_trait {
                    "└── "
                } else {
                    "├── "
                };
                let child_prefix = if is_last_trait { "    " } else { "│   " };

                tree_out.push_str(&format!(
                    "{}{} {}\n",
                    prefix.blue(),
                    "χαρακτήρ".magenta(),
                    name.clone().bold()
                ));

                for (j, method) in methods.iter().enumerate() {
                    let is_last_method = j == methods.len() - 1;
                    let method_prefix = if is_last_method {
                        "└── "
                    } else {
                        "├── "
                    };

                    let params_str = method
                        .params
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, t))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let ret_str = method
                        .return_type
                        .as_ref()
                        .map(|t| format!(" -> {}", t))
                        .unwrap_or_default();

                    tree_out.push_str(&format!(
                        "{}{} Ἔργον {}({}){}\n",
                        child_prefix.blue(),
                        method_prefix.blue(),
                        method.name.green(),
                        params_str.yellow(),
                        ret_str.yellow()
                    ));
                }
            }
        }
    }

    writeln!(writer, "{}", tree_out).into_diagnostic()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typetree_output() {
        let source = "
            εἶδος Χρήστης ὁρίζειν {
                ὄνομα ὀνόματος.
                ἡλικία ἀριθμοῦ.
            }.
        ";
        let mut buffer = Vec::new();
        run_typetree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Internally types might be lowercased depending on normalization in Scope
        assert!(output.contains("Χρήστης") || output.contains("χρηστης"));
        assert!(output.contains("ὄνομα") || output.contains("ονομα"));
        assert!(output.contains("ἡλικία") || output.contains("ηλικια"));
        assert!(output.contains("├──") || output.contains("└──"));
    }
}
