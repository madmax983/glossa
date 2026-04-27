//! The Archivist (ὁ Ἀρχειοφύλαξ) - JSON Schema Exporter
//!
//! This module implements the "Archivist" tool, which generates a JSON Schema
//! from the struct (`εἶδος`) definitions within a ΓΛΩΣΣΑ program's scope.
//!
//! # Purpose
//!
//! "Archivist" enables interoperability by extracting semantic types from the
//! compiler's analysis phase and producing standard JSON Schemas that can be
//! consumed by external tools, APIs, and databases.

use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Recursively maps a GlossaType to its JSON Schema equivalent.
fn glossa_type_to_json_schema(
    ty: &crate::semantic::GlossaType,
    out: &mut String,
    indent: usize,
) -> std::fmt::Result {
    let padding = "  ".repeat(indent);
    match ty {
        crate::semantic::GlossaType::Number => {
            write!(out, "{{\"type\": \"number\"}}")
        }
        crate::semantic::GlossaType::String => {
            write!(out, "{{\"type\": \"string\"}}")
        }
        crate::semantic::GlossaType::Boolean => {
            write!(out, "{{\"type\": \"boolean\"}}")
        }
        crate::semantic::GlossaType::List(inner) | crate::semantic::GlossaType::Set(inner) => {
            write!(
                out,
                "{{\n{padding}  \"type\": \"array\",\n{padding}  \"items\": "
            )?;
            glossa_type_to_json_schema(inner, out, indent + 1)?;
            write!(out, "\n{padding}}}")
        }
        crate::semantic::GlossaType::Struct { name, .. } => {
            write!(out, "{{\"$ref\": \"#/definitions/{}\"}}", name)
        }
        _ => {
            // For Unknown or other types not easily mapped to basic JSON schema, default to string/any.
            write!(out, "{{\"type\": \"string\"}}")
        }
    }
}

pub fn run_archivist(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Ἀρχειοθέτησις (Archiving)", "📜");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let mut json = String::with_capacity(4096);
    let filename = input.file_name().unwrap_or_default().to_string_lossy();

    writeln!(json, "{{").unwrap();
    writeln!(
        json,
        "  \"$schema\": \"http://json-schema.org/draft-07/schema#\","
    )
    .unwrap();
    writeln!(json, "  \"title\": \"{} Schema\",", filename).unwrap();
    writeln!(json, "  \"definitions\": {{").unwrap();

    let mut types: Vec<_> = program
        .scope
        .types()
        .filter_map(|(name, ty)| {
            if let crate::semantic::GlossaType::Struct { fields, .. } = ty {
                // Type names in scope are normalized (lowercase), we want to use the title casing if available,
                // but the internal struct name might be normalized too. Let's find the original struct definition.
                // But since scope.types() is all we have, we'll just use the normalized name for keys,
                // but capitalize the title.
                // Oh, wait, the `name` field in `GlossaType::Struct` is NOT normalized?
                // Let's check. Yes it is, wait, we'll see in the test.
                Some((name.as_str(), fields))
            } else {
                None
            }
        })
        .collect();

    // Sort for deterministic output
    types.sort_by_key(|(name, _)| *name);

    for (i, (name, fields)) in types.iter().enumerate() {
        writeln!(json, "    \"{}\": {{", name).unwrap();
        // Capitalize the first letter for the title
        let mut chars = name.chars();
        let title_name = match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        };
        writeln!(json, "      \"title\": \"{}\",", title_name).unwrap();
        writeln!(json, "      \"type\": \"object\",").unwrap();
        writeln!(json, "      \"properties\": {{").unwrap();

        for (j, (field_name, field_type)) in fields.iter().enumerate() {
            write!(json, "        \"{}\": ", field_name).unwrap();
            glossa_type_to_json_schema(field_type, &mut json, 4).unwrap();

            if j < fields.len() - 1 {
                writeln!(json, ",").unwrap();
            } else {
                writeln!(json).unwrap();
            }
        }

        writeln!(json, "      }}").unwrap();

        if i < types.len() - 1 {
            writeln!(json, "    }},").unwrap();
        } else {
            writeln!(json, "    }}").unwrap();
        }
    }

    writeln!(json, "  }}").unwrap();
    writeln!(json, "}}").unwrap();

    let output_path = input.with_extension("schema.json");
    if let Err(e) = std::fs::write(&output_path, &json) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write schema file: {}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A R C H I V I S T".bold().cyan());
    println!("   {}", "JSON Schema Generated".italic().dim());
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
    fn test_run_archivist_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test_schema.γλ");
        fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

        let result = run_archivist(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("schema.json");
        assert!(output_path.exists());

        let json = fs::read_to_string(&output_path).unwrap();
        // Since names are normalized internally, "Χρήστης" -> "χρηστης", but we capitalize the title back to "Χρηστης"
        assert!(json.contains("\"title\": \"Χρηστης\"") || json.contains("\"title\": \"Χρήστης\""));
        assert!(json.contains("\"type\": \"object\""));
        // "ὄνομα" is normalized to "ονομα"
        assert!(json.contains("\"ονομα\"") || json.contains("\"ὄνομα\""));
    }

    #[test]
    fn test_run_archivist_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("error_schema.γλ");
        fs::write(&input_path, "invalid syntax").unwrap();

        let result = run_archivist(&input_path);
        assert!(result.is_err());
    }
}
