//! The Architect (ὁ Ἀρχιτέκτων) - JSON Schema Generator
//!
//! This module implements the "Architect" tool, which inspects type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

pub fn run_architect(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    println!("📐 Ἀρχιτέκτων (Generating JSON Schema)...");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let mut schemas = Vec::new();

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            let mut output = String::new();
            let _ = writeln!(output, "{{");
            let _ = writeln!(
                output,
                "  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\","
            );
            let _ = writeln!(output, "  \"title\": \"{}\",", name);
            let _ = writeln!(output, "  \"type\": \"object\",");
            let _ = writeln!(output, "  \"properties\": {{");
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let json_type = glossa_type_to_json_schema(field_type);
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(
                    output,
                    "    \"{}\": {{ {} }}{}",
                    field_name, json_type, comma
                );
            }
            let _ = writeln!(output, "  }},");

            let required_fields: Vec<String> = fields
                .iter()
                .filter(|(_, t)| !matches!(t, GlossaType::Option(_)))
                .map(|(n, _)| format!("\"{}\"", n))
                .collect();

            if !required_fields.is_empty() {
                let _ = writeln!(output, "  \"required\": [{}]", required_fields.join(", "));
            } else {
                let _ = writeln!(output, "  \"required\": []");
            }
            let _ = writeln!(output, "}}");
            schemas.push(output);
        }
    }

    println!();
    println!("Γ Λ Ω Σ Σ Α   A R C H I T E C T");
    println!("JSON Schema");
    println!();

    for schema in schemas {
        println!("```json");
        println!("{}", schema.trim());
        println!("```\n");
    }

    Ok(())
}

fn glossa_type_to_json_schema(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "\"type\": \"integer\"".to_string(),
        GlossaType::String => "\"type\": \"string\"".to_string(),
        GlossaType::Boolean => "\"type\": \"boolean\"".to_string(),
        GlossaType::List(inner) => format!(
            "\"type\": \"array\", \"items\": {{ {} }}",
            glossa_type_to_json_schema(inner)
        ),
        GlossaType::Set(inner) => format!(
            "\"type\": \"array\", \"uniqueItems\": true, \"items\": {{ {} }}",
            glossa_type_to_json_schema(inner)
        ),
        GlossaType::Map(_, _) => "\"type\": \"object\"".to_string(),
        GlossaType::Option(inner) => glossa_type_to_json_schema(inner),
        _ => "\"type\": \"object\"".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema() {
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Number),
            "\"type\": \"integer\""
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::String),
            "\"type\": \"string\""
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Boolean),
            "\"type\": \"boolean\""
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Unknown),
            "\"type\": \"object\""
        );
    }
}

    #[test]
    fn test_glossa_type_to_json_schema_complex() {
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            "\"type\": \"array\", \"items\": { \"type\": \"integer\" }"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Set(Box::new(GlossaType::String))),
            "\"type\": \"array\", \"uniqueItems\": true, \"items\": { \"type\": \"string\" }"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            "\"type\": \"object\""
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Option(Box::new(GlossaType::Boolean))),
            "\"type\": \"boolean\""
        );
    }
