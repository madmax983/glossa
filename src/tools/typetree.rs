//! Terminal Type Tree Visualizer
//!
//! This module provides the `glossa typetree` command, which visualizes
//! the nested structures of `εἶδος` (Structs) and `χαρακτήρ` (Traits)
//! using an ASCII/Unicode hierarchy tree.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, GlossaType, analyze_program};
use crate::tools::ui::Status;
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::path::Path;

/// Run the TypeTree tool on a file
pub fn run_typetree(input_path: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Δένδρον Εἰδῶν (TypeTree)", "🌳");

    let source = crate::tools::runner::load_source(input_path)?;

    let mut buffer = Vec::new();
    run_typetree_inner(&source, &mut buffer)?;
    let output = String::from_utf8(buffer).into_diagnostic()?;

    status.success();

    println!();
    use crossterm::style::Stylize;
    println!("   {}", "Γ Λ Ω Σ Σ Α   T Y P E T R E E".bold().cyan());
    println!("   {}", "Semantic Data Model Hierarchy".italic().dim());
    println!();
    print!("{}", output);

    Ok(())
}

/// Internal implementation of TypeTree logic
pub fn run_typetree_inner<W: std::io::Write>(source: &str, writer: &mut W) -> Result<()> {
    let ast = parse(source)?;
    let program = analyze_program(&ast)?;

    let tree = generate_tree(&program);
    write!(writer, "{}", tree).into_diagnostic()?;

    Ok(())
}

fn generate_tree(program: &AnalyzedProgram) -> String {
    let mut tree = String::new();
    let mut processed = HashSet::new();

    // Group types
    let mut structs = Vec::new();
    let mut traits = Vec::new();

    for (name, ty) in program.scope.get_all_types() {
        if let GlossaType::Struct { .. } = ty {
            structs.push((name, ty));
        }
    }

    for (name, trait_def) in program.scope.get_all_traits() {
        traits.push((name, trait_def));
    }

    // Sort to ensure deterministic output
    structs.sort_by(|a: &(smol_str::SmolStr, GlossaType), b| a.0.cmp(&b.0));
    traits.sort_by(|a: &(smol_str::SmolStr, crate::semantic::model::TraitDef), b| a.0.cmp(&b.0));

    if structs.is_empty() && traits.is_empty() {
        return "No types or traits found in the program.\n".to_string();
    }

    if !structs.is_empty() {
        tree.push_str("Εἴδη (Structs):\n");
        for (name, ty) in structs {
            if let GlossaType::Struct { fields, .. } = ty {
                tree.push_str(&format!("εἶδος {}\n", name));
                let field_count = fields.len();
                for (i, (field_name, field_type)) in fields.iter().enumerate() {
                    let is_last = i == field_count - 1;
                    let prefix = if is_last { "└── " } else { "├── " };
                    tree.push_str(&format!("{}{}: {}\n", prefix, field_name, field_type));

                    // Recursively print nested structs if they are complex
                    let child_prefix = if is_last { "    " } else { "│   " };
                    print_nested_type(&mut tree, field_type, child_prefix, &mut processed);
                }
                tree.push('\n');
            }
        }
    }

    if !traits.is_empty() {
        tree.push_str("Χαρακτῆρες (Traits):\n");
        for (name, trait_def) in traits {
            tree.push_str(&format!("χαρακτήρ {}\n", name));
            let method_count = trait_def.methods.len();
            for (i, method) in trait_def.methods.iter().enumerate() {
                let is_last = i == method_count - 1;
                let prefix = if is_last { "└── " } else { "├── " };

                let params_str: Vec<String> = method
                    .params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect();

                let ret_str = method
                    .return_type
                    .as_ref()
                    .map(|t| format!(" -> {}", t))
                    .unwrap_or_default();

                tree.push_str(&format!(
                    "{}fn {}({}){}\n",
                    prefix,
                    method.name,
                    params_str.join(", "),
                    ret_str
                ));
            }
            tree.push('\n');
        }
    }

    tree
}

fn print_nested_type(
    tree: &mut String,
    ty: &GlossaType,
    prefix: &str,
    processed: &mut HashSet<String>,
) {
    match ty {
        GlossaType::List(inner) | GlossaType::Set(inner) | GlossaType::Option(inner) => {
            print_nested_type(tree, inner, prefix, processed);
        }
        GlossaType::Map(k, v) | GlossaType::Result(k, v) => {
            print_nested_type(tree, k, prefix, processed);
            print_nested_type(tree, v, prefix, processed);
        }
        GlossaType::Struct { name, fields, .. } => {
            // Prevent infinite recursion on self-referential structs
            if processed.contains(name.as_str()) {
                tree.push_str(&format!("{}... (recursive {}) \n", prefix, name));
                return;
            }
            processed.insert(name.to_string());

            let field_count = fields.len();
            if field_count > 0 {
                for (i, (field_name, field_type)) in fields.iter().enumerate() {
                    let is_last = i == field_count - 1;
                    let branch = if is_last { "└── " } else { "├── " };
                    tree.push_str(&format!(
                        "{}{}{}: {}\n",
                        prefix, branch, field_name, field_type
                    ));

                    let child_prefix =
                        format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                    print_nested_type(tree, field_type, &child_prefix, processed);
                }
            }

            processed.remove(name.as_str());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typetree_basic_struct() {
        let source = "
            εἶδος Χρήστης ὁρίζειν {
                ὄνομα ὀνόματος.
                ἡλικία ἀριθμοῦ.
            }.
        ";
        let mut buffer = Vec::new();
        run_typetree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("εἶδος χρηστης") || output.contains("εἶδος Χρήστης"));
        assert!(output.contains("├── ονομα: Ὄνομα") || output.contains("├── ὄνομα: Ὄνομα"));
        assert!(output.contains("└── ηλικια: Ἀριθμός") || output.contains("└── ἡλικία: Ἀριθμός"));
    }

    #[test]
    fn test_typetree_nested_struct() {
        let source = "
            εἶδος Διεύθυνσις ὁρίζειν {
                πόλις ὀνόματος.
            }.
            εἶδος Χρήστης ὁρίζειν {
                διεύθυνσις Διεύθυνσις.
            }.
        ";
        let mut buffer = Vec::new();
        run_typetree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Print output to help debug if it fails
        println!("Output:\n{}", output);

        assert!(
            output.contains("└── διευθυνσις: Εἶδος") || output.contains("└── διεύθυνσις: Εἶδος")
        );
        assert!(output.contains("    └── πολις: Ὄνομα") || output.contains("    └── πόλις: Ὄνομα"));
    }

    #[test]
    fn test_typetree_trait() {
        let source = "
            χαρακτήρ Εκτυπώσιμος ὁρίζειν {
                δεῖ τυπωσις τῷ self.
            }.
        ";
        let mut buffer = Vec::new();
        run_typetree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("χαρακτήρ εκτυπωσιμος") || output.contains("χαρακτήρ Εκτυπώσιμος"));
        assert!(output.contains("└── fn τυπωσις(self: Ἄγνωστον)"));
    }

    #[test]
    fn test_run_typetree_empty() {
        let source = "ξ πέντε ἔστω.";
        let mut buffer = Vec::new();
        run_typetree_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("No types or traits found in the program."));
    }
}
