//! The Merchant (ὁ Ἔμπορος) - JSON Schema Exporter
//!
//! This module implements the "Merchant" tool, providing an exporter that converts
//! ΓΛΩΣΣΑ type definitions (`εἶδος`) into standard JSON Schema descriptions.
//!
//! # Purpose
//!
//! Merchants trade goods across different lands. The Merchant tool trades data
//! structures across different programming ecosystems by generating JSON Schema
//! definitions from Glossa structs, enabling easy integration with REST APIs and other tools.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Generates a JSON Schema representation for a given Glossa type.
pub fn write_glossa_type_to_json_schema(g_type: &GlossaType, out: &mut String) -> std::fmt::Result {
    match g_type {
        GlossaType::Number => write!(out, r#"{{"type": "integer"}}"#),
        GlossaType::String => write!(out, r#"{{"type": "string"}}"#),
        GlossaType::Boolean => write!(out, r#"{{"type": "boolean"}}"#),
        GlossaType::List(inner) => {
            write!(out, r#"{{"type": "array", "items": "#)?;
            write_glossa_type_to_json_schema(inner, out)?;
            write!(out, r#"}}"#)
        }
        GlossaType::Set(inner) => {
            write!(out, r#"{{"type": "array", "uniqueItems": true, "items": "#)?;
            write_glossa_type_to_json_schema(inner, out)?;
            write!(out, r#"}}"#)
        }
        GlossaType::Map(_key, value) => {
            write!(out, r#"{{"type": "object", "additionalProperties": "#)?;
            // JSON schema only supports string keys, but we document the value type
            write_glossa_type_to_json_schema(value, out)?;
            write!(out, r#"}}"#)
        }
        GlossaType::Option(inner) => {
            write!(out, r#"{{"anyOf": [{{"type": "null"}}, "#)?;
            write_glossa_type_to_json_schema(inner, out)?;
            write!(out, r#"]}}"#)
        }
        GlossaType::Result(ok, _) => {
            // A simplified Result representation just showing the ok type
            write_glossa_type_to_json_schema(ok, out)
        }
        GlossaType::Struct { name, fields, .. } => {
            write!(
                out,
                r#"{{"type": "object", "title": "{}", "properties": {{"#,
                name
            )?;
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                write!(out, r#""{}": "#, field_name)?;
                write_glossa_type_to_json_schema(field_type, out)?;
                if i < fields.len() - 1 {
                    write!(out, ", ")?;
                }
            }
            // All fields are required by default in Glossa (unless Option)
            let required_fields: Vec<String> = fields
                .iter()
                .filter(|(_, t)| !matches!(t, GlossaType::Option(_)))
                .map(|(n, _)| format!(r#""{}""#, n))
                .collect();

            write!(out, r#"}}"#)?;
            if !required_fields.is_empty() {
                write!(out, r#", "required": [{}]"#, required_fields.join(", "))?;
            }
            write!(out, r#"}}"#)
        }
        _ => write!(out, r#"{{"type": "object"}}"#),
    }
}

/// Runs the Merchant tool on a given Glossa source file.
pub fn run_merchant(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἔμπορος (Generating JSON Schema)", "📦");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    status.success();

    let mut output = String::new();
    let mut schemas_found = false;

    // Collect all type definitions
    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            schemas_found = true;
            let g_type = GlossaType::Struct {
                name: name.clone(),
                gender: crate::morphology::Gender::Masculine, // dummy gender for export
                fields: fields.clone(),
            };

            let mut schema_json = String::new();
            write_glossa_type_to_json_schema(&g_type, &mut schema_json)
                .expect("Failed to format JSON");

            writeln!(output, "{}", name.bold().green()).unwrap();
            writeln!(output, "{}", schema_json).unwrap();
            writeln!(output).unwrap();
        }
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   M E R C H A N T".bold().yellow());
    println!("   {}", "JSON Schema Exporter".italic().dim());
    println!();

    if schemas_found {
        println!("{}", output);
    } else {
        println!("  {}", "Οὐδὲν εἶδος εὑρέθη (No types found).".dim());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_glossa_type_to_json_schema_basic_types() {
        let mut out = String::new();

        write_glossa_type_to_json_schema(&GlossaType::Number, &mut out).unwrap();
        assert_eq!(out, r#"{"type": "integer"}"#);
        out.clear();

        write_glossa_type_to_json_schema(&GlossaType::String, &mut out).unwrap();
        assert_eq!(out, r#"{"type": "string"}"#);
        out.clear();

        write_glossa_type_to_json_schema(&GlossaType::Boolean, &mut out).unwrap();
        assert_eq!(out, r#"{"type": "boolean"}"#);
    }

    #[test]
    fn test_write_glossa_type_to_json_schema_complex_types() {
        let mut out = String::new();

        write_glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number)), &mut out)
            .unwrap();
        assert_eq!(out, r#"{"type": "array", "items": {"type": "integer"}}"#);
        out.clear();

        write_glossa_type_to_json_schema(
            &GlossaType::Option(Box::new(GlossaType::String)),
            &mut out,
        )
        .unwrap();
        assert_eq!(out, r#"{"anyOf": [{"type": "null"}, {"type": "string"}]}"#);
    }
}

#[test]
fn test_write_glossa_type_to_json_schema_set_map_result() {
    let mut out = String::new();

    write_glossa_type_to_json_schema(&GlossaType::Set(Box::new(GlossaType::Number)), &mut out)
        .unwrap();
    assert_eq!(
        out,
        r#"{"type": "array", "uniqueItems": true, "items": {"type": "integer"}}"#
    );
    out.clear();

    write_glossa_type_to_json_schema(
        &GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
        &mut out,
    )
    .unwrap();
    assert_eq!(
        out,
        r#"{"type": "object", "additionalProperties": {"type": "integer"}}"#
    );
    out.clear();

    write_glossa_type_to_json_schema(
        &GlossaType::Result(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
        &mut out,
    )
    .unwrap();
    assert_eq!(out, r#"{"type": "string"}"#);
}

#[test]
fn test_write_glossa_type_to_json_schema_struct() {
    let mut out = String::new();

    let struct_type = GlossaType::Struct {
        name: smol_str::SmolStr::new("User"),
        gender: crate::morphology::Gender::Masculine,
        fields: vec![
            (smol_str::SmolStr::new("id"), GlossaType::Number),
            (smol_str::SmolStr::new("name"), GlossaType::String),
            (
                smol_str::SmolStr::new("age"),
                GlossaType::Option(Box::new(GlossaType::Number)),
            ),
        ],
    };

    write_glossa_type_to_json_schema(&struct_type, &mut out).unwrap();
    assert_eq!(
        out,
        r#"{"type": "object", "title": "User", "properties": {"id": {"type": "integer"}, "name": {"type": "string"}, "age": {"anyOf": [{"type": "null"}, {"type": "integer"}]}}, "required": ["id", "name"]}"#
    );
}

#[test]
fn test_write_glossa_type_to_json_schema_struct_no_required() {
    let mut out = String::new();

    let struct_type = GlossaType::Struct {
        name: smol_str::SmolStr::new("User"),
        gender: crate::morphology::Gender::Masculine,
        fields: vec![(
            smol_str::SmolStr::new("age"),
            GlossaType::Option(Box::new(GlossaType::Number)),
        )],
    };

    write_glossa_type_to_json_schema(&struct_type, &mut out).unwrap();
    assert_eq!(
        out,
        r#"{"type": "object", "title": "User", "properties": {"age": {"anyOf": [{"type": "null"}, {"type": "integer"}]}}}"#
    );
}

#[test]
fn test_write_glossa_type_to_json_schema_fallback() {
    let mut out = String::new();
    write_glossa_type_to_json_schema(&GlossaType::Unknown, &mut out).unwrap();
    assert_eq!(out, r#"{"type": "object"}"#);
}
