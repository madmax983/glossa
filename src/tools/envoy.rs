//! The Envoy (ὁ Πρέσβυς) - JSON Schema Generator
//!
//! This module implements the "Envoy" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema formats.
//!
//! # Purpose
//!
//! An envoy represents a sovereign entity to foreign nations. This tool translates
//! internal ΓΛΩΣΣΑ structures into a universal format (JSON Schema), allowing
//! data contracts to be shared with and validated by external systems and languages.
//!
//! # How it Works
//!
//! The [`run_envoy`](crate::tools::envoy::run_envoy) function orchestrates the process:
//! 1. Parses and semantically analyzes the source code.
//! 2. Scans the Abstract Syntax Tree for `TypeDefinition` nodes.
//! 3. Maps ΓΛΩΣΣΑ types (`ἀριθμοῦ`, `ὀνόματος`, etc.) to JSON Schema representations.
//! 4. Outputs formatted JSON to the terminal using a stylish, colorful table.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Envoy tool to generate JSON Schemas from Glossa types.
///
/// The Envoy (Πρέσβυς) tool reads the provided source file, compiles it, and automatically
/// generates corresponding JSON Schema documents for any type definitions found.
pub fn run_envoy(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Πρέσβυς (Generating JSON Schema)", "🌍");

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
            let mut schema = String::new();
            let _ = writeln!(schema, "{{");
            let _ = writeln!(
                schema,
                "  \"$schema\": \"http://json-schema.org/draft-07/schema#\","
            );
            let _ = writeln!(schema, "  \"title\": \"{}\",", name);
            let _ = writeln!(schema, "  \"type\": \"object\",");
            let _ = writeln!(schema, "  \"properties\": {{");

            let mut required = Vec::new();

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let (type_json, is_required) = glossa_type_to_json_schema(field_type);
                if is_required {
                    required.push(field_name);
                }

                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(schema, "    \"{}\": {}{}", field_name, type_json, comma);
            }

            let _ = writeln!(schema, "  }},");

            if !required.is_empty() {
                let _ = write!(schema, "  \"required\": [");
                for (i, r) in required.iter().enumerate() {
                    if i > 0 {
                        let _ = write!(schema, ", ");
                    }
                    let _ = write!(schema, "\"{}\"", r);
                }
                let _ = writeln!(schema, "]");
            } else {
                let _ = writeln!(schema, "  \"required\": []");
            }

            let _ = writeln!(schema, "}}");

            output.push_str(&schema);
            output.push('\n');
        }
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   E N V O Y".bold().cyan());
    println!("   {}", "JSON Schema".italic().dim());
    println!();

    if output.is_empty() {
        println!("   {}", "No types found to generate schemas for.".dim());
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("JSON Schema")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    let formatted_code = format!("```json\n{}\n```", output.trim());
    table.add_row(vec![Cell::new(formatted_code)]);

    println!("{table}");
    println!();

    Ok(())
}

fn glossa_type_to_json_schema(g_type: &GlossaType) -> (String, bool) {
    match g_type {
        GlossaType::Number => ("{\"type\": \"integer\"}".to_string(), true),
        GlossaType::String => ("{\"type\": \"string\"}".to_string(), true),
        GlossaType::Boolean => ("{\"type\": \"boolean\"}".to_string(), true),
        GlossaType::List(inner) => {
            let (inner_json, _) = glossa_type_to_json_schema(inner);
            (
                format!("{{\"type\": \"array\", \"items\": {}}}", inner_json),
                true,
            )
        }
        GlossaType::Set(inner) => {
            let (inner_json, _) = glossa_type_to_json_schema(inner);
            (
                format!(
                    "{{\"type\": \"array\", \"uniqueItems\": true, \"items\": {}}}",
                    inner_json
                ),
                true,
            )
        }
        GlossaType::Map(_, _) => (
            "{\"type\": \"object\", \"additionalProperties\": true}".to_string(),
            true,
        ),
        GlossaType::Option(inner) => {
            let (inner_json, _) = glossa_type_to_json_schema(inner);
            (inner_json, false) // Not required
        }
        _ => ("{}".to_string(), true), // Any type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema() {
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Number),
            ("{\"type\": \"integer\"}".to_string(), true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::String),
            ("{\"type\": \"string\"}".to_string(), true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Boolean),
            ("{\"type\": \"boolean\"}".to_string(), true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            (
                "{\"type\": \"array\", \"items\": {\"type\": \"integer\"}}".to_string(),
                true
            )
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Set(Box::new(GlossaType::Number))),
            (
                "{\"type\": \"array\", \"uniqueItems\": true, \"items\": {\"type\": \"integer\"}}"
                    .to_string(),
                true
            )
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Option(Box::new(GlossaType::Number))),
            ("{\"type\": \"integer\"}".to_string(), false)
        );
    }
}
