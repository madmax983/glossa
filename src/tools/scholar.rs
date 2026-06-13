//! The Scholar (ὁ Σχολαστικός) - API Doc Generator
//!
//! This module implements the "Scholar" tool, which parses a ΓΛΩΣΣΑ program
//! and automatically generates comprehensive Markdown API documentation (`doc.md`).
//!
//! # The "Missing" Link
//!
//! Why does this module exist? In the modern software era, if a library isn't
//! documented, it doesn't exist. The Scholar bridges the gap between raw semantic
//! analysis (the Abstract Syntax Tree) and human-readable references. Instead of
//! forcing developers to read Ancient Greek source files to understand an API's
//! shape, the Scholar distills the program's defined structures (`εἴδη`), traits
//! (`χαρακτῆρες`), and functions (`ἔργα`) into a clean, GitHub-flavored Markdown file.
//!
//! # How it Works
//!
//! 1. Parses the target `.γλ` file.
//! 2. Extracts type definitions (Structs), traits (Interfaces), and verbs (Functions).
//! 3. Formats them into standardized Markdown tables and headers.
//! 4. Saves the result alongside the original file.

use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Scholar tool to generate Markdown documentation from Glossa code.
///
/// This function acts as the entry point for the documentation generator. It takes
/// the path to a source file, triggers the semantic analyzer, and writes the
/// resulting API documentation to `{input_filename}.doc.md`.
///
/// ## Examples
///
/// ```rust,no_run
/// use glossa::tools::scholar::run_scholar;
/// use std::path::Path;
///
/// let input = Path::new("library.γλ");
/// if let Err(e) = run_scholar(&input) {
///     eprintln!("Documentation generation failed: {}", e);
/// }
fn write_types_docs(md: &mut String, program: &crate::semantic::AnalyzedProgram) {
    let mut types = program.scope.types().peekable();
    if types.peek().is_some() {
        writeln!(md, "## Types (Εἴδη)\n").unwrap();
        for (name, type_def) in types {
            writeln!(md, "### `{}`\n", name).unwrap();
            if let crate::semantic::GlossaType::Struct { fields, .. } = type_def {
                if !fields.is_empty() {
                    writeln!(md, "| Field | Type |\n|-------|------|").unwrap();
                    for (field_name, field_type) in fields {
                        writeln!(md, "| `{}` | `{}` |", field_name, field_type).unwrap();
                    }
                    md.push('\n');
                } else {
                    writeln!(md, "*No fields defined.*\n").unwrap();
                }
            }
        }
    }
}

fn write_traits_docs(md: &mut String, program: &crate::semantic::AnalyzedProgram) {
    let mut traits = program.scope.traits().peekable();
    if traits.peek().is_some() {
        writeln!(md, "## Traits (Χαρακτῆρες)\n").unwrap();
        for (name, trait_def) in traits {
            writeln!(md, "### `{}`\n", name).unwrap();
            if !trait_def.methods.is_empty() {
                for method in &trait_def.methods {
                    writeln!(md, "* `{}`", method.name).unwrap();
                }
                md.push('\n');
            } else {
                writeln!(md, "*No methods defined.*\n").unwrap();
            }
        }
    }
}

fn write_functions_docs(md: &mut String, program: &crate::semantic::AnalyzedProgram) {
    let mut functions = program.scope.functions().peekable();
    if functions.peek().is_some() {
        writeln!(md, "## Functions (Ἔργα)\n").unwrap();
        for func in functions {
            // ⚡ Bolt Optimization: Use `write!` to build strings dynamically without intermediate `Vec` collections.
            write!(md, "### `{}(", func.name).unwrap();
            for (i, t) in func.param_types.iter().enumerate() {
                if i > 0 {
                    write!(md, ", ").unwrap();
                }
                write!(md, "{}", t).unwrap();
            }
            write!(md, ") -> ").unwrap();
            if let Some(ret_type) = &func.return_type {
                write!(md, "{}", ret_type).unwrap();
            } else {
                write!(md, "Οὐδέν").unwrap();
            }
            writeln!(md, "`\n").unwrap();
        }
    }
}

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

    let mut md = String::with_capacity(4096);
    let filename = input.file_name().unwrap_or_default().to_string_lossy();

    writeln!(md, "# API Documentation: `{}`\n", filename).unwrap();

    write_types_docs(&mut md, &program);
    write_traits_docs(&mut md, &program);
    write_functions_docs(&mut md, &program);

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

    #[test]
    fn test_run_scholar_file_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");
        fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { }.").unwrap();

        // Create a directory at the expected output path so that fs::write fails
        let output_path = input_path.with_extension("doc.md");
        fs::create_dir(&output_path).unwrap();

        let result = run_scholar(&input_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to write")
                || err_msg.contains("directory")
                || err_msg.contains("denied")
                || err_msg.contains("Permission")
        );
    }

    #[test]
    fn test_run_scholar_empty_fields_methods_functions() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");

        let source = "
        εἶδος Χρήστης ὁρίζειν { }.
        χαρακτήρ Εὐγενής ὁρίζειν { }.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_scholar(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("doc.md");
        assert!(output_path.exists());

        let md = fs::read_to_string(&output_path).unwrap();
        assert!(md.contains("*No fields defined.*"));
        assert!(md.contains("*No methods defined.*"));
    }

    #[test]
    fn test_run_scholar_with_functions() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");

        let source = "
        προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ· δός ξ ψ ἄθροισμα.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_scholar(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("doc.md");
        assert!(output_path.exists());

        let md = fs::read_to_string(&output_path).unwrap();
        assert!(md.contains("### `προσθεσις(Ἀριθμός, Ἀριθμός) -> Ἀριθμός`"));
    }
}
