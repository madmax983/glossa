//! The Diplomat (ὁ Διπλωμάτης) - JSON Schema Generator
//!
//! This module implements the "Diplomat" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema (Draft 7) objects.
//!
//! # Purpose
//!
//! A diplomat negotiates terms between foreign lands. This tool bridges the gap
//! between the ancient structures of ΓΛΩΣΣΑ and modern data exchange formats (JSON)
//! by generating universal schemas for validation.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Diplomat tool to generate JSON Schemas from Glossa types.
///
/// The Diplomat (Διπλωμάτης) tool reads the provided source file, compiles it, and automatically
/// generates a combined JSON Schema representing all type definitions found.
pub fn run_diplomat(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Διπλωμάτης (Generating JSON Schema)", "🤝");

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
    let _ = writeln!(output, "{{");
    let _ = writeln!(
        output,
        "  \"$schema\": \"http://json-schema.org/draft-07/schema#\","
    );
    let _ = writeln!(output, "  \"definitions\": {{");

    let mut has_defs = false;
    let mut defs = Vec::new();

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            has_defs = true;
            let mut def_str = String::new();
            let _ = writeln!(def_str, "    \"{}\": {{", name);
            let _ = writeln!(def_str, "      \"type\": \"object\",");
            let _ = writeln!(def_str, "      \"properties\": {{");

            let mut required_fields = Vec::new();
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let schema_type = glossa_type_to_json_schema(field_type);
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(
                    def_str,
                    "        \"{}\": {}{}",
                    field_name, schema_type, comma
                );

                if !is_optional(field_type) {
                    required_fields.push(format!("\"{}\"", field_name));
                }
            }
            let _ = writeln!(def_str, "      }}");
            if !required_fields.is_empty() {
                let _ = writeln!(
                    def_str,
                    "      ,\"required\": [{}]",
                    required_fields.join(", ")
                );
            }
            let _ = write!(def_str, "    }}");
            defs.push(def_str);
        }
    }

    if has_defs {
        output.push_str(&defs.join(",\n"));
        output.push('\n');
    }

    let _ = writeln!(output, "  }}");
    let _ = writeln!(output, "}}");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   D I P L O M A T".bold().cyan());
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

fn is_optional(g_type: &GlossaType) -> bool {
    matches!(g_type, GlossaType::Option(_))
}

fn glossa_type_to_json_schema(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "{\"type\": \"integer\"}".to_string(),
        GlossaType::String => "{\"type\": \"string\"}".to_string(),
        GlossaType::Boolean => "{\"type\": \"boolean\"}".to_string(),
        GlossaType::List(inner) => {
            let inner_schema = glossa_type_to_json_schema(inner);
            format!("{{\"type\": \"array\", \"items\": {}}}", inner_schema)
        }
        GlossaType::Set(inner) => {
            let inner_schema = glossa_type_to_json_schema(inner);
            format!(
                "{{\"type\": \"array\", \"uniqueItems\": true, \"items\": {}}}",
                inner_schema
            )
        }
        GlossaType::Map(_, _) => "{\"type\": \"object\"}".to_string(),
        GlossaType::Option(inner) => glossa_type_to_json_schema(inner),
        GlossaType::Struct { name, .. } => format!("{{\"$ref\": \"#/definitions/{}\"}}", name),
        _ => "{}".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema() {
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Number),
            "{\"type\": \"integer\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::String),
            "{\"type\": \"string\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Boolean),
            "{\"type\": \"boolean\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            "{\"type\": \"array\", \"items\": {\"type\": \"integer\"}}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Set(Box::new(GlossaType::Number))),
            "{\"type\": \"array\", \"uniqueItems\": true, \"items\": {\"type\": \"integer\"}}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            "{\"type\": \"object\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Option(Box::new(GlossaType::Number))),
            "{\"type\": \"integer\"}"
        );
        assert_eq!(glossa_type_to_json_schema(&GlossaType::Unknown), "{}");
    }
}
