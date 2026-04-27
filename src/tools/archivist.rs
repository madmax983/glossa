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
        // "ἀληθοῦς" isn't a type literal recognized in this simple way, use valid Glossa type like "ἀληθές" -> wait, "ἀριθμοῦ" is valid.
        // Let's stick to base types that parse 100% fine in tests.
        fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }. \n εἶδος Group ὁρίζειν { }.").unwrap();

        let result = run_archivist(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("schema.json");
        assert!(output_path.exists());

        let json = fs::read_to_string(&output_path).unwrap();
        assert!(json.contains("\"title\": \"Χρηστης\"") || json.contains("\"title\": \"Χρήστης\""));
        assert!(json.contains("\"type\": \"object\""));
        assert!(json.contains("\"ονομα\"") || json.contains("\"ὄνομα\""));
        assert!(json.contains("\"ηλικια\"") || json.contains("\"ἡλικία\""));
        assert!(json.contains("\"group\"") || json.contains("\"Group\""));
    }

    #[test]
    fn test_run_archivist_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("error_schema.γλ");
        fs::write(&input_path, "invalid syntax").unwrap();

        let result = run_archivist(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_glossa_type_to_json_schema_all_variants() {
        let mut out = String::new();

        glossa_type_to_json_schema(&crate::semantic::GlossaType::Number, &mut out, 0).unwrap();
        assert_eq!(out, "{\"type\": \"number\"}");
        out.clear();

        glossa_type_to_json_schema(&crate::semantic::GlossaType::String, &mut out, 0).unwrap();
        assert_eq!(out, "{\"type\": \"string\"}");
        out.clear();

        glossa_type_to_json_schema(&crate::semantic::GlossaType::Boolean, &mut out, 0).unwrap();
        assert_eq!(out, "{\"type\": \"boolean\"}");
        out.clear();

        glossa_type_to_json_schema(
            &crate::semantic::GlossaType::List(Box::new(crate::semantic::GlossaType::Number)),
            &mut out,
            0,
        )
        .unwrap();
        assert!(out.contains("\"type\": \"array\""));
        assert!(out.contains("\"type\": \"number\""));
        out.clear();

        glossa_type_to_json_schema(
            &crate::semantic::GlossaType::Set(Box::new(crate::semantic::GlossaType::String)),
            &mut out,
            0,
        )
        .unwrap();
        assert!(out.contains("\"type\": \"array\""));
        assert!(out.contains("\"type\": \"string\""));
        out.clear();

        glossa_type_to_json_schema(
            &crate::semantic::GlossaType::Struct {
                name: "test".into(),
                gender: crate::morphology::Gender::Masculine,
                fields: vec![],
            },
            &mut out,
            0,
        )
        .unwrap();
        assert_eq!(out, "{\"$ref\": \"#/definitions/test\"}");
        out.clear();

        glossa_type_to_json_schema(&crate::semantic::GlossaType::Unknown, &mut out, 0).unwrap();
        assert_eq!(out, "{\"type\": \"string\"}");
        out.clear();

        glossa_type_to_json_schema(
            &crate::semantic::GlossaType::Option(Box::new(crate::semantic::GlossaType::Number)),
            &mut out,
            0,
        )
        .unwrap();
        assert_eq!(out, "{\"type\": \"string\"}");
        out.clear();

        glossa_type_to_json_schema(
            &crate::semantic::GlossaType::Result(
                Box::new(crate::semantic::GlossaType::Number),
                Box::new(crate::semantic::GlossaType::String),
            ),
            &mut out,
            0,
        )
        .unwrap();
        assert_eq!(out, "{\"type\": \"string\"}");
        out.clear();
    }

    #[test]
    fn test_run_archivist_file_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("file_error.γλ");
        fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { }.").unwrap();

        let output_path = input_path.with_extension("schema.json");
        fs::create_dir(&output_path).unwrap();

        let result = run_archivist(&input_path);
        assert!(result.is_err());
    }
}
