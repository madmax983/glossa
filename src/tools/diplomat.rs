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

    let output = generate_schema_string(&program);

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

pub(crate) fn generate_schema_string(program: &crate::semantic::AnalyzedProgram) -> String {
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
    output
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
    use std::io::Write as IoWrite;

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

        let set_type = GlossaType::Set(Box::new(GlossaType::Number));
        assert_eq!(
            glossa_type_to_json_schema(&set_type),
            "{\"type\": \"array\", \"items\": {\"type\": \"integer\"}}"
        );

        let option_type = GlossaType::Option(Box::new(GlossaType::Number));
        assert_eq!(
            glossa_type_to_json_schema(&option_type),
            "{\"type\": [\"integer\", \"null\"]}"
        );

        let map_type = GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number));
        assert_eq!(
            glossa_type_to_json_schema(&map_type),
            "{\"type\": \"object\", \"additionalProperties\": {\"type\": \"integer\"}}"
        );

        let complex_option = GlossaType::Option(Box::new(GlossaType::Map(
            Box::new(GlossaType::String),
            Box::new(GlossaType::Number),
        )));
        assert_eq!(
            glossa_type_to_json_schema(&complex_option),
            "{\"type\": [\"object\", \"null\"], \"additionalProperties\": {\"type\": \"integer\"}}"
        );

        let unknown_option = GlossaType::Option(Box::new(GlossaType::Unknown));
        assert_eq!(
            glossa_type_to_json_schema(&unknown_option),
            "{\"anyOf\": [{}, {\"type\": \"null\"}]}"
        );

        assert_eq!(glossa_type_to_json_schema(&GlossaType::Unknown), "{}");
    }

    #[test]
    fn test_generate_schema_string_complex() {
        use crate::semantic::{AnalyzedProgram, Scope};
        use smol_str::SmolStr;

        let stmt1 = AnalyzedStatement::TypeDefinition {
            name: SmolStr::new("StructA"),
            fields: vec![
                (SmolStr::new("req_field"), GlossaType::Number),
                (SmolStr::new("opt_field"), GlossaType::Option(Box::new(GlossaType::String))),
            ],
        };

        let stmt2 = AnalyzedStatement::TypeDefinition {
            name: SmolStr::new("StructB"),
            fields: vec![],
        };

        let program = AnalyzedProgram {
            statements: vec![stmt1, stmt2],
            scope: Scope::new(),
        };

        let output = generate_schema_string(&program);

        // Assert it handled the comma between multiple structs
        assert!(output.contains("\"StructA\": {"));
        assert!(output.contains("},\n    \"StructB\": {"));

        // Assert it handled Option types correctly for "required" properties list
        assert!(output.contains("\"req_field\""));
        assert!(!output.contains("\"opt_field\"\n      ]")); // Should not be in required array
    }

    #[test]
    fn test_run_diplomat_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test_schema.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }. \n".as_bytes())
                .unwrap();
        }

        let result = run_diplomat(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diplomat_no_structs() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("no_structs.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε» λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_diplomat(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diplomat_errors() {
        // Test file not found error path
        let result = run_diplomat(Path::new("nonexistent.gl"));
        assert!(result.is_err());

        // Test syntax/semantic error path
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("error.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("invalid syntax that fails analysis\n".as_bytes())
                .unwrap();
        }
        let result = run_diplomat(&input_path);
        assert!(result.is_err());
    }
}
