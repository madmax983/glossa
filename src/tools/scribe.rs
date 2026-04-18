//! The Scribe Tool (ὁ Γραμματεύς)
//!
//! This module implements the "Scribe" functionality, an experimental tool
//! that reads a ΓΛΩΣΣΑ program and generates an API reference in Markdown format.
//! It extracts structs, functions, and traits defined in the file.

use crate::semantic::GlossaType;
use crate::tools::runner::analyze_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::Path;

/// Generates a Markdown API reference for a given ΓΛΩΣΣΑ file
pub fn run_scribe(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Γράφων (Scribing)", "📜");

    // Check size to prevent OOM on massive files
    if let Ok(metadata) = std::fs::metadata(input) {
        let is_too_large = metadata.len() > 1024 * 1024;
        if is_too_large {
            status.error("Σφάλμα (Error)");
            return Err(miette::miette!("Ἀρχεῖον λίαν μέγα: {}", input.display()));
        }
    }

    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(miette::miette!("{}", e));
        }
    };

    let program = match analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let mut md = String::new();
    let filename = input.file_name().unwrap_or_default().to_string_lossy();
    md.push_str(&format!("# API Reference: `{}`\n\n", filename));

    // 1. Types (Structs)
    let mut types: Vec<_> = program.scope.types().collect();
    types.sort_by(|a, b| a.0.cmp(b.0)); // Sort by name

    if !types.is_empty() {
        md.push_str("## 🏛️ Εἴδη (Types)\n\n");
        for (name, ty) in types {
            if let GlossaType::Struct { fields, .. } = ty {
                md.push_str(&format!("### `{}`\n\n", name));
                if fields.is_empty() {
                    md.push_str("*No fields.*\n\n");
                } else {
                    md.push_str("| Field | Type |\n");
                    md.push_str("|---|---|\n");
                    for (f_name, f_type) in fields {
                        md.push_str(&format!("| `{}` | `{}` |\n", f_name, f_type));
                    }
                    md.push('\n');
                }
            } else {
                md.push_str(&format!("### `{}`\n\n*Alias for `{}`*\n\n", name, ty));
            }
        }
    }

    // 2. Traits
    let mut traits: Vec<_> = program.scope.traits().collect();
    traits.sort_by(|a, b| a.0.cmp(b.0));

    if !traits.is_empty() {
        md.push_str("## 🎭 Χαρακτῆρες (Traits)\n\n");
        for (name, def) in traits {
            md.push_str(&format!("### `{}`\n\n", name));
            if def.methods.is_empty() {
                md.push_str("*No methods.*\n\n");
            } else {
                for method in &def.methods {
                    let mut signature = format!("ἔργον {}(", method.name);
                    for (i, (p_name, p_type)) in method.params.iter().enumerate() {
                        if i > 0 {
                            signature.push_str(", ");
                        }
                        signature.push_str(&format!("{}: {}", p_name, p_type));
                    }
                    signature.push(')');

                    if let Some(ret) = &method.return_type {
                        signature.push_str(&format!(" -> {}", ret));
                    }

                    md.push_str(&format!("- `{}`\n", signature));
                }
                md.push('\n');
            }
        }
    }

    // 3. Functions
    let mut functions: Vec<_> = program.scope.functions().collect();
    functions.sort_by(|a, b| a.name.cmp(&b.name));

    if !functions.is_empty() {
        md.push_str("## ⚡ Ἔργα (Functions)\n\n");
        for func in functions {
            let mut signature = format!("ἔργον {}(", func.name);
            for (i, p_type) in func.param_types.iter().enumerate() {
                if i > 0 {
                    signature.push_str(", ");
                }
                signature.push_str(&format!("P{}: {}", i, p_type));
            }
            signature.push(')');

            if let Some(ret) = &func.return_type {
                signature.push_str(&format!(" -> {}", ret));
            }

            md.push_str(&format!("- `{}`\n", signature));
        }
        md.push('\n');
    }

    // Write output
    let mut output_filename = filename.into_owned();
    if output_filename.ends_with(".γλ") {
        output_filename = output_filename.replace(".γλ", "_doc.md");
    } else {
        output_filename.push_str("_doc.md");
    }

    let output_path = input.with_file_name(output_filename);

    if let Err(e) = fs::write(&output_path, &md).into_diagnostic() {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(e);
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S C R I B E".bold().cyan());
    println!("   {}", "API Reference Generated".italic().dim());
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
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_run_scribe_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test_lib.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            let source = "
            εἶδος Χρήστης ὁρίζειν {
                ὄνομα ὀνόματος.
                ἡλικία ἀριθμοῦ.
            }.

            χαρακτήρ Ὁμιλητής ὁρίζειν {
                λάλει ὁρίζειν (α ὀνόματος).
            }.

            πρόσθεσις ὁρίζειν τῷ α ἀριθμοῦ τῷ β ἀριθμοῦ · α β ἄθροισμα δός.
            ";
            f.write_all(source.as_bytes()).unwrap();
        }

        let result = run_scribe(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_file_name("test_lib_doc.md");
        assert!(output_path.exists());

        let md = std::fs::read_to_string(&output_path).unwrap();
        assert!(md.contains("# API Reference: `test_lib.γλ`"));
        assert!(md.contains("## 🏛️ Εἴδη (Types)"));
        assert!(md.contains("### `Χρήστης`"));
        assert!(md.contains("`ὄνομα`"));
        assert!(md.contains("`ἡλικία`"));

        assert!(md.contains("## 🎭 Χαρακτῆρες (Traits)"));
        assert!(md.contains("### `Ὁμιλητής`"));
        assert!(md.contains("ἔργον λάλει"));

        assert!(md.contains("## ⚡ Ἔργα (Functions)"));
        assert!(md.contains("`ἔργον πρόσθεσις"));
    }

    #[test]
    fn test_run_scribe_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_scribe(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_scribe_parse_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_scribe(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_scribe_file_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("too_large.γλ");

        // Create a file larger than 1MB
        let max_size = 1024 * 1024;
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            let data = vec![0u8; max_size + 1];
            f.write_all(&data).unwrap();
        }

        let result = run_scribe(&input_path);
        assert!(result.is_err());
    }
}
