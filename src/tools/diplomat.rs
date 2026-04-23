//! The Diplomat (ὁ Διπλωμάτης) - JSON Schema Generator
//!
//! This module implements the "Diplomat" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema representations.
//!
//! # Purpose
//!
//! Connecting ΓΛΩΣΣΑ logic to external APIs and configurations is essential for
//! real-world systems. JSON is the language of external systems. The Diplomat establishes
//! formal agreements (schemas) between our Greek models and the outside world.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Run the Diplomat tool to generate JSON Schema representations from Glossa types.
pub fn run_diplomat(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Διπλωμάτης (Generating JSON Schema)", "📜");

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
    output.push_str("{\n");
    output.push_str("  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n");
    output.push_str("  \"definitions\": {\n");

    let mut first_struct = true;
    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            if !first_struct {
                output.push_str(",\n");
            }
            first_struct = false;

            let _ = writeln!(output, "    \"{}\": {{", name);
            output.push_str("      \"type\": \"object\",\n");
            output.push_str("      \"properties\": {\n");

            let mut required_fields = Vec::new();

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let json_type = glossa_type_to_json_schema(field_type);
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(output, "        \"{}\": {}{}", field_name, json_type, comma);

                // If it is not an Option type, it is required
                if !matches!(field_type, GlossaType::Option(_)) {
                    required_fields.push(field_name.clone());
                }
            }

            output.push_str("      }");

            if !required_fields.is_empty() {
                output.push_str(",\n      \"required\": [\n");
                for (i, req) in required_fields.iter().enumerate() {
                    let comma = if i < required_fields.len() - 1 {
                        ","
                    } else {
                        ""
                    };
                    let _ = writeln!(output, "        \"{}\"{}", req, comma);
                }
                output.push_str("      ]\n");
            } else {
                output.push('\n');
            }

            output.push_str("    }");
        }
    }

    if first_struct {
        // No structs found
        output.push('\n');
    } else {
        output.push('\n');
    }

    output.push_str("  }\n");
    output.push_str("}\n");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   D I P L O M A T".bold().cyan());
    println!("   {}", "JSON Schema Definitions".italic().dim());
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

fn glossa_type_to_json_schema(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "{\"type\": \"integer\"}".to_string(),
        GlossaType::String => "{\"type\": \"string\"}".to_string(),
        GlossaType::Boolean => "{\"type\": \"boolean\"}".to_string(),
        GlossaType::List(inner) | GlossaType::Set(inner) => {
            format!(
                "{{\"type\": \"array\", \"items\": {}}}",
                glossa_type_to_json_schema(inner)
            )
        }
        GlossaType::Map(_, inner) => {
            format!(
                "{{\"type\": \"object\", \"additionalProperties\": {}}}",
                glossa_type_to_json_schema(inner)
            )
        }
        GlossaType::Option(inner) => {
            // A simple Draft-07 compatible representation for nullable types:
            let inner_json = glossa_type_to_json_schema(inner);

            // If the inner object starts with {"type": "something", ...}
            // replace it with {"type": ["something", "null"], ...}
            if inner_json.starts_with("{\"type\": \"") {
                let end_of_type = inner_json
                    .find("\",")
                    .unwrap_or(inner_json.find("\"}").unwrap_or(0));
                if end_of_type > 0 {
                    let type_str = &inner_json[10..end_of_type];
                    return format!(
                        "{{\"type\": [\"{}\", \"null\"]{}",
                        type_str,
                        &inner_json[end_of_type + 1..]
                    );
                }
            }

            // Fallback: anyOf
            format!("{{\"anyOf\": [{}, {{\"type\": \"null\"}}]}}", inner_json)
        }
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

        let list_type = GlossaType::List(Box::new(GlossaType::String));
        assert_eq!(
            glossa_type_to_json_schema(&list_type),
            "{\"type\": \"array\", \"items\": {\"type\": \"string\"}}"
        );

        let option_type = GlossaType::Option(Box::new(GlossaType::Number));
        assert_eq!(
            glossa_type_to_json_schema(&option_type),
            "{\"type\": [\"integer\", \"null\"]}"
        );

        assert_eq!(glossa_type_to_json_schema(&GlossaType::Unknown), "{}");
    }
}
