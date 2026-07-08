//! The Herald (ὁ Κῆρυξ) - JSON Schema Generator
//!
//! This module implements the "Herald" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema definitions.
//!
//! # Purpose
//!
//! The Herald connects ΓΛΩΣΣΑ structures to modern APIs. By transpiling
//! `εἶδος` directly into standard JSON Schema, it makes ancient types
//! instantly usable for modern data interchange and validation.
//!
//! # How it Works
//!
//! The [`run_herald`](crate::tools::herald::run_herald) function:
//! 1. Parses and semantically analyzes the source code.
//! 2. Scans the Abstract Syntax Tree for `TypeDefinition` nodes.
//! 3. Maps ΓΛΩΣΣΑ types to JSON Schema equivalents.
//! 4. Outputs formatted JSON schema to the terminal.
use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Herald tool to generate JSON schemas from Glossa types.
///
/// # Errors
/// Returns a [`miette::Result`] if the file cannot be read, or if there is a parsing
/// or semantic analysis error during compilation.
pub fn run_herald(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Κῆρυξ (Generating JSON Schema)", "📢");

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

    let mut schemas = Vec::new();

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            let mut schema = String::new();
            let _ = writeln!(schema, "    \"{}\": {{", name);
            let _ = writeln!(schema, "      \"type\": \"object\",");
            let _ = writeln!(schema, "      \"properties\": {{");

            let mut required_fields = Vec::new();

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let field_schema = glossa_type_to_json_schema(field_type);

                if !matches!(field_type, GlossaType::Option(_)) {
                    required_fields.push(format!("\"{}\"", field_name));
                }

                let _ = writeln!(
                    schema,
                    "        \"{}\": {}{}",
                    field_name, field_schema, comma
                );
            }

            let _ = writeln!(schema, "      }},");

            if !required_fields.is_empty() {
                let _ = writeln!(
                    schema,
                    "      \"required\": [{}]",
                    required_fields.join(", ")
                );
            } else {
                // remove the comma from the properties object end if there are no required fields.
                // a bit of a hack but it's fine for this scale
                if schema.ends_with(",\n") {
                    schema.pop();
                    schema.pop();
                    schema.push('\n');
                }
            }

            let _ = write!(schema, "    }}");
            schemas.push(schema);
        }
    }

    let mut output = String::new();
    let _ = writeln!(output, "{{");
    let _ = writeln!(
        output,
        "  \"$schema\": \"http://json-schema.org/draft-07/schema#\","
    );
    let _ = writeln!(output, "  \"definitions\": {{");

    for (i, schema) in schemas.iter().enumerate() {
        let comma = if i < schemas.len() - 1 { ",\n" } else { "\n" };
        let _ = write!(output, "{}{}", schema, comma);
    }

    let _ = writeln!(output, "  }}");
    let _ = writeln!(output, "}}");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   H E R A L D".bold().cyan());
    println!("   {}", "JSON Schema".italic().dim());
    println!();

    println!("{}", output.trim());
    println!();

    Ok(())
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
        GlossaType::Map(_, value_type) => {
            let value_schema = glossa_type_to_json_schema(value_type);
            format!(
                "{{\"type\": \"object\", \"additionalProperties\": {}}}",
                value_schema
            )
        }
        GlossaType::Option(inner) => {
            glossa_type_to_json_schema(inner) // The 'required' logic handles the optionality
        }
        GlossaType::Struct { name, .. } => {
            format!("{{\"$ref\": \"#/definitions/{}\"}}", name)
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
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            "{\"type\": \"array\", \"items\": {\"type\": \"integer\"}}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Option(Box::new(GlossaType::Number))),
            "{\"type\": \"integer\"}"
        );
    }
}
