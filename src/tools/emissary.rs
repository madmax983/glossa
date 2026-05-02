//! The Emissary (ὁ Ἀπεσταλμένος) - JSON Schema Exporter
//!
//! This module implements an experimental exporter that converts ΓΛΩΣΣΑ `εἶδος`
//! definitions into standard JSON Schema documents.
//!
//! # Purpose
//!
//! The Emissary bridges the gap between the ancient world of ΓΛΩΣΣΑ and modern web APIs.
//! By automatically generating standard JSON Schemas from internal structs, it allows
//! external systems to safely validate payloads before they even reach the ΓΛΩΣΣΑ backend.

use crate::semantic::{AnalyzedStatement, GlossaType};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Generate the JSON Schema for a set of fields
fn generate_json_schema(name: &str, fields: &[(smol_str::SmolStr, GlossaType)]) -> String {
    let mut schema = String::new();
    let _ = write!(
        schema,
        "{{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n"
    );
    let _ = writeln!(schema, "  \"title\": \"{}\",", name);
    let _ = writeln!(schema, "  \"type\": \"object\",");
    let _ = writeln!(schema, "  \"properties\": {{");

    let mut required_fields = Vec::new();

    for (i, (field_name, field_type)) in fields.iter().enumerate() {
        let is_last = i == fields.len() - 1;
        let comma = if is_last { "" } else { "," };

        match field_type {
            GlossaType::Option(_) => {}
            _ => required_fields.push(field_name.to_string()),
        }

        let type_mapping: std::borrow::Cow<'_, str> = match field_type {
            GlossaType::Number => std::borrow::Cow::Borrowed("\"type\": \"integer\""),
            GlossaType::String => std::borrow::Cow::Borrowed("\"type\": \"string\""),
            GlossaType::Boolean => std::borrow::Cow::Borrowed("\"type\": \"boolean\""),
            GlossaType::List(inner) | GlossaType::Set(inner) => {
                let inner_type = match **inner {
                    GlossaType::Number => "integer",
                    GlossaType::String => "string",
                    GlossaType::Boolean => "boolean",
                    _ => "object", // Fallback for complex inner types
                };
                std::borrow::Cow::Owned(format!(
                    "\"type\": \"array\", \"items\": {{ \"type\": \"{}\" }}",
                    inner_type
                ))
            }
            GlossaType::Option(inner) => {
                let inner_type = match **inner {
                    GlossaType::Number => "integer",
                    GlossaType::String => "string",
                    GlossaType::Boolean => "boolean",
                    _ => "object", // Fallback for complex inner types
                };
                // JSON Schema doesn't strictly have an "optional" type, it uses the "required" array.
                // We just output the inner type here.
                std::borrow::Cow::Owned(format!("\"type\": \"{}\"", inner_type))
            }
            _ => std::borrow::Cow::Borrowed("\"type\": \"object\""), // Fallback for Struct, Map, Function, etc.
        };

        let _ = writeln!(
            schema,
            "    \"{}\": {{ {} }}{}",
            field_name, type_mapping, comma
        );
    }

    let _ = write!(schema, "  }}");

    if !required_fields.is_empty() {
        let _ = write!(schema, ",\n  \"required\": [\n");
        for (i, req) in required_fields.iter().enumerate() {
            let is_last = i == required_fields.len() - 1;
            let comma = if is_last { "" } else { "," };
            let _ = writeln!(schema, "    \"{}\"{}", req, comma);
        }
        let _ = writeln!(schema, "  ]");
    } else {
        let _ = writeln!(schema);
    }

    let _ = write!(schema, "}}");
    schema
}

/// Run the Emissary tool on a file
pub fn run_emissary(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status =
        crate::tools::ui::Status::start_with_symbol("Ἀπεσταλμένος (Generating JSON Schemas)", "📤");

    let source = match crate::tools::runner::load_source(input) {
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

    let mut schemas_generated = 0;
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Struct (εἶδος)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("JSON Schema")
            .add_attribute(Attribute::Bold)
            .fg(Color::Green),
    ]);

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            let schema = generate_json_schema(name, fields);
            table.add_row(vec![
                Cell::new(name)
                    .fg(Color::Cyan)
                    .add_attribute(Attribute::Bold),
                Cell::new(schema).fg(Color::DarkGrey),
            ]);
            schemas_generated += 1;
        }
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   E M I S S A R Y".bold().cyan());
    println!("   {}", "JSON Schema Exporter".italic().dim());
    println!();

    if schemas_generated == 0 {
        println!("   {}", "No type definitions found.".italic().dim());
    } else {
        println!("{table}");
        println!(
            "   Generated schemas for {} types.",
            schemas_generated.to_string().cyan()
        );
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_json_schema_basic() {
        let fields = vec![
            ("name".into(), GlossaType::String),
            ("age".into(), GlossaType::Number),
            ("is_active".into(), GlossaType::Boolean),
        ];

        let schema = generate_json_schema("User", &fields);

        assert!(schema.contains("\"title\": \"User\""));
        assert!(schema.contains("\"type\": \"object\""));
        assert!(schema.contains("\"name\": { \"type\": \"string\" }"));
        assert!(schema.contains("\"age\": { \"type\": \"integer\" }"));
        assert!(schema.contains("\"is_active\": { \"type\": \"boolean\" }"));

        // All these are required because they are not Option
        assert!(schema.contains("\"required\": ["));
        assert!(schema.contains("\"name\","));
        assert!(schema.contains("\"age\","));
        assert!(schema.contains("\"is_active\""));
    }

    #[test]
    fn test_generate_json_schema_collections_and_options() {
        let fields = vec![
            (
                "tags".into(),
                GlossaType::List(Box::new(GlossaType::String)),
            ),
            (
                "nickname".into(),
                GlossaType::Option(Box::new(GlossaType::String)),
            ),
        ];

        let schema = generate_json_schema("Profile", &fields);

        assert!(
            schema
                .contains("\"tags\": { \"type\": \"array\", \"items\": { \"type\": \"string\" } }")
        );
        assert!(schema.contains("\"nickname\": { \"type\": \"string\" }"));

        // Only tags should be required, nickname is optional
        assert!(schema.contains("\"required\": ["));
        assert!(schema.contains("\"tags\""));
        // Need to check that nickname isn't in the required array specifically,
        // because it IS in the properties dictionary, making `!schema.contains("\"nickname\"")` fail.
        let required_section = schema.split("\"required\": [").nth(1).unwrap();
        assert!(!required_section.contains("\"nickname\""));
    }
}
