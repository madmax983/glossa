//! The Nomothetes (ὁ Νομοθέτης) - JSON Schema Generator
//!
//! This module implements the "Nomothetes" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema format.
//!
//! # Purpose
//!
//! A Nomothetes was an ancient Greek lawmaker. In the digital realm, JSON Schema
//! is the law that governs data shapes. This tool allows ΓΛΩΣΣΑ to act as the
//! single source of truth for API contracts, generating standard JSON Schema
//! that can be consumed by frontend applications, validators, and other services.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Nomothetes tool to generate JSON Schemas from Glossa types.
///
/// The Nomothetes (Νομοθέτης) reads the provided source file, compiles it, and
/// generates standard JSON Schema definitions for any `εἶδος` (structs) found.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::nomothetes::run_nomothetes;
/// use std::path::Path;
///
/// let input = Path::new("schema.γλ");
/// if let Err(e) = run_nomothetes(&input) {
///     eprintln!("Schema generation failed: {}", e);
/// }
/// ```
pub fn run_nomothetes(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Νομοθέτης (Generating JSON Schema)", "⚖️");

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

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            let mut properties = Vec::new();
            let mut required = Vec::new();

            for (field_name, field_type) in fields {
                let prop_json = glossa_type_to_json_schema(field_type);
                properties.push(format!(r#"      "{}": {}"#, field_name, prop_json));

                if !matches!(field_type, GlossaType::Option(_)) {
                    required.push(format!(r#""{}""#, field_name));
                }
            }

            let properties_json = properties.join(",\n");
            let required_json = required.join(", ");

            let req_block = if !required.is_empty() {
                format!(
                    r#",
    "required": [{}]"#,
                    required_json
                )
            } else {
                "".to_string()
            };

            let schema = format!(
                r#"{{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "{}",
  "type": "object",
  "properties": {{
{}
  }}{}
}}"#,
                name, properties_json, req_block
            );

            let _ = writeln!(output, "--- {} Schema ---\n{}\n", name, schema);
        }
    }

    if output.is_empty() {
        let _ = writeln!(output, "No types found in the program.");
    }

    println!();
    println!(
        "   {}",
        "Γ Λ Ω Σ Σ Α   N O M O T H E T E S".bold().magenta()
    );
    println!("   {}", "JSON Schema Generator".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("JSON Schema")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
    ]);

    let formatted_code = format!("```json\n{}\n```", output.trim());
    table.add_row(vec![Cell::new(formatted_code)]);

    println!("{table}");
    println!();

    Ok(())
}

fn glossa_type_to_json_schema(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => r#"{"type": "integer"}"#.to_string(),
        GlossaType::String => r#"{"type": "string"}"#.to_string(),
        GlossaType::Boolean => r#"{"type": "boolean"}"#.to_string(),
        GlossaType::List(inner) => {
            let inner_schema = glossa_type_to_json_schema(inner);
            format!(r#"{{"type": "array", "items": {}}}"#, inner_schema)
        }
        GlossaType::Set(inner) => {
            let inner_schema = glossa_type_to_json_schema(inner);
            format!(
                r#"{{"type": "array", "uniqueItems": true, "items": {}}}"#,
                inner_schema
            )
        }
        GlossaType::Map(_, v) => {
            let v_schema = glossa_type_to_json_schema(v);
            format!(
                r#"{{"type": "object", "additionalProperties": {}}}"#,
                v_schema
            )
        }
        GlossaType::Option(inner) => {
            let inner_schema = glossa_type_to_json_schema(inner);
            format!(r#"{{"anyOf": [{{"type": "null"}}, {}]}}"#, inner_schema)
        }
        _ => r#"{"type": "object"}"#.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema() {
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Number),
            r#"{"type": "integer"}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::String),
            r#"{"type": "string"}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Boolean),
            r#"{"type": "boolean"}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            r#"{"type": "array", "items": {"type": "integer"}}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Set(Box::new(GlossaType::Number))),
            r#"{"type": "array", "uniqueItems": true, "items": {"type": "integer"}}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            r#"{"type": "object", "additionalProperties": {"type": "integer"}}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Option(Box::new(GlossaType::Number))),
            r#"{"anyOf": [{"type": "null"}, {"type": "integer"}]}"#
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Unknown),
            r#"{"type": "object"}"#
        );
    }
}
