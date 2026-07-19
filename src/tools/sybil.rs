//! The Sybil (ἡ Σίβυλλα) - JSON Schema Exporter
//!
//! This module implements the "Sybil" tool, which inspects type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema.
//!
//! # Purpose
//!
//! The Sybil Oracle foretold truths in structured patterns. This tool
//! enables interoperability by generating standard JSON Schemas from
//! Glossa struct definitions, bridging the gap between ancient syntax
//! and modern web APIs.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Sybil tool to generate JSON Schemas from Glossa types.
pub fn run_sybil(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Σίβυλλα (Generating JSON Schema)", "👁️");

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
            let mut required_fields = Vec::new();
            let mut properties = String::new();

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let (json_type, is_required) = glossa_type_to_json_schema(field_type);
                if is_required {
                    required_fields.push(format!("\"{}\"", field_name));
                }

                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(
                    properties,
                    "        \"{}\": {{ \"type\": \"{}\" }}{}",
                    field_name, json_type, comma
                );
            }

            let required_str = if required_fields.is_empty() {
                "".to_string()
            } else {
                format!(",\n    \"required\": [{}]", required_fields.join(", "))
            };

            let _ = writeln!(
                output,
                "{{\n    \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n    \"title\": \"{}\",\n    \"type\": \"object\",\n    \"properties\": {{\n{}    }}{}\n}}\n",
                name,
                properties.trim_end(),
                required_str
            );
        }
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S Y B I L".bold().cyan());
    println!("   {}", "JSON Schema".italic().dim());
    println!();

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

fn glossa_type_to_json_schema(g_type: &GlossaType) -> (&'static str, bool) {
    match g_type {
        GlossaType::Number => ("integer", true),
        GlossaType::String => ("string", true),
        GlossaType::Boolean => ("boolean", true),
        GlossaType::List(_) | GlossaType::Set(_) => ("array", true),
        GlossaType::Option(inner) => {
            let (inner_type, _) = glossa_type_to_json_schema(inner);
            (inner_type, false)
        }
        _ => ("object", true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema() {
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Number),
            ("integer", true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::String),
            ("string", true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Boolean),
            ("boolean", true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            ("array", true)
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Option(Box::new(GlossaType::String))),
            ("string", false)
        );
    }
}
