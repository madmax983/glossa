//! The Emissary (ὁ Ἀπεσταλμένος) - JSON Schema Generator
//!
//! This module implements the "Emissary" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema representations.
//!
//! # Purpose
//!
//! While Papyrus allows integration with relational databases, JSON Schema is the lingua franca
//! of modern APIs and document stores. The Emissary acts as a diplomatic envoy, translating
//! the strict, ancient structural definitions of ΓΛΩΣΣΑ into a format the modern web understands.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Emissary tool to generate JSON Schemas from Glossa types.
///
/// The Emissary (Ἀπεσταλμένος) tool reads the provided source file, compiles it, and automatically
/// generates corresponding JSON Schema representations for any type definitions found.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::emissary::run_emissary;
/// use std::path::Path;
///
/// let input = Path::new("schema.γλ");
/// if let Err(e) = run_emissary(&input) {
///     eprintln!("Schema generation failed: {}", e);
/// }
/// ```
///
/// # Errors
///
/// Returns a [`miette::Result`] if the file cannot be read, or if there is a parsing
/// or semantic analysis error during compilation.
pub fn run_emissary(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀπεσταλμένος (Generating JSON Schema)", "📜");

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

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let mut type_schema = String::new();
                glossa_type_to_json_schema(field_type, &mut type_schema, 4);
                let _ = writeln!(
                    schema,
                    "    \"{}\": {}{}",
                    field_name,
                    type_schema.trim(),
                    comma
                );
            }

            let _ = writeln!(schema, "  }},");
            let _ = write!(schema, "  \"required\": [");

            let required_fields: Vec<_> = fields
                .iter()
                .filter(|(_, ty)| !matches!(ty, GlossaType::Option(_)))
                .map(|(name, _)| format!("\"{}\"", name))
                .collect();

            let _ = write!(schema, "{}", required_fields.join(", "));
            let _ = writeln!(schema, "]");
            let _ = writeln!(schema, "}}");

            output.push_str(&schema);
            output.push('\n');
        }
    }

    if output.is_empty() {
        println!("No type definitions found.");
        return Ok(());
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   E M I S S A R Y".bold().cyan());
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

fn glossa_type_to_json_schema(g_type: &GlossaType, out: &mut String, indent: usize) {
    let spaces = " ".repeat(indent);
    match g_type {
        GlossaType::Number => {
            let _ = write!(
                out,
                "{}{{\n{}  \"type\": \"integer\"\n{}}}",
                spaces, spaces, spaces
            );
        }
        GlossaType::String => {
            let _ = write!(
                out,
                "{}{{\n{}  \"type\": \"string\"\n{}}}",
                spaces, spaces, spaces
            );
        }
        GlossaType::Boolean => {
            let _ = write!(
                out,
                "{}{{\n{}  \"type\": \"boolean\"\n{}}}",
                spaces, spaces, spaces
            );
        }
        GlossaType::List(inner) | GlossaType::Set(inner) => {
            let _ = writeln!(out, "{}{{", spaces);
            let _ = writeln!(out, "{}  \"type\": \"array\",", spaces);
            let _ = writeln!(out, "{}  \"items\": ", spaces);
            let mut inner_schema = String::new();
            glossa_type_to_json_schema(inner, &mut inner_schema, indent + 4);
            let _ = writeln!(out, "{}", inner_schema.trim_end());
            let _ = write!(out, "{}}}", spaces);
        }
        GlossaType::Map(_key, value) => {
            let _ = writeln!(out, "{}{{", spaces);
            let _ = writeln!(out, "{}  \"type\": \"object\",", spaces);
            let _ = writeln!(out, "{}  \"additionalProperties\": ", spaces);
            let mut val_schema = String::new();
            glossa_type_to_json_schema(value, &mut val_schema, indent + 4);
            let _ = writeln!(out, "{}", val_schema.trim_end());
            let _ = write!(out, "{}}}", spaces);
        }
        GlossaType::Option(inner) => {
            glossa_type_to_json_schema(inner, out, indent);
        }
        _ => {
            let _ = write!(out, "{}{{}}", spaces);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema_number() {
        let mut out = String::new();
        glossa_type_to_json_schema(&GlossaType::Number, &mut out, 0);
        assert_eq!(out, "{\n  \"type\": \"integer\"\n}");
    }

    #[test]
    fn test_glossa_type_to_json_schema_string() {
        let mut out = String::new();
        glossa_type_to_json_schema(&GlossaType::String, &mut out, 0);
        assert_eq!(out, "{\n  \"type\": \"string\"\n}");
    }

    #[test]
    fn test_glossa_type_to_json_schema_boolean() {
        let mut out = String::new();
        glossa_type_to_json_schema(&GlossaType::Boolean, &mut out, 0);
        assert_eq!(out, "{\n  \"type\": \"boolean\"\n}");
    }

    #[test]
    fn test_glossa_type_to_json_schema_list() {
        let mut out = String::new();
        glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number)), &mut out, 0);
        let expected = "{\n  \"type\": \"array\",\n  \"items\": \n    {\n      \"type\": \"integer\"\n    }\n}";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_glossa_type_to_json_schema_map() {
        let mut out = String::new();
        glossa_type_to_json_schema(
            &GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number)),
            &mut out,
            0,
        );
        let expected = "{\n  \"type\": \"object\",\n  \"additionalProperties\": \n    {\n      \"type\": \"integer\"\n    }\n}";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_glossa_type_to_json_schema_option() {
        let mut out = String::new();
        glossa_type_to_json_schema(
            &GlossaType::Option(Box::new(GlossaType::Number)),
            &mut out,
            0,
        );
        assert_eq!(out, "{\n  \"type\": \"integer\"\n}");
    }

    #[test]
    fn test_glossa_type_to_json_schema_unknown() {
        let mut out = String::new();
        glossa_type_to_json_schema(&GlossaType::Unknown, &mut out, 0);
        assert_eq!(out, "{}");
    }
}
