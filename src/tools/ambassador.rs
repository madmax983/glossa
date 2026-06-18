//! The Ambassador (ὁ Πρέσβυς) - JSON Schema Generator
//!
//! This module implements the "Ambassador" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema.
//!
//! # Purpose
//!
//! The Ambassador represents the Glossa kingdom in the realm of JSON. It translates
//! our internal definitions into the widely accepted JSON Schema format, enabling
//! interoperability with other systems and APIs.

use crate::parser::parse;
use crate::semantic::{AnalyzedStatement, GlossaType, analyze_program};
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Ambassador tool to generate JSON Schema from Glossa types.
pub fn run_ambassador(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Πρέσβυς (Generating JSON Schema)", "📜");

    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(miette::miette!("Failed to read file: {}", e));
        }
    };

    let ast = match parse(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα ἀναγνώσεως (Parse Error)");
            return Err(e.into());
        }
    };

    let program = match analyze_program(&ast) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e.into());
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

    let mut first_struct = true;
    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            if !first_struct {
                let _ = writeln!(output, ",");
            }
            first_struct = false;

            let _ = writeln!(output, "    \"{}\": {{", name);
            let _ = writeln!(output, "      \"type\": \"object\",");
            let _ = writeln!(output, "      \"properties\": {{");

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let mut schema_str = String::new();
                glossa_type_to_json_schema(field_type, &mut schema_str, 8);

                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(
                    output,
                    "        \"{}\": {}{}",
                    field_name,
                    schema_str.trim_end(),
                    comma
                );
            }
            let _ = writeln!(output, "      }},");

            let required_fields: Vec<_> = fields
                .iter()
                .filter_map(|(n, t)| {
                    if matches!(t, GlossaType::Option(_)) {
                        None
                    } else {
                        Some(n.as_str())
                    }
                })
                .collect();

            if !required_fields.is_empty() {
                let _ = writeln!(output, "      \"required\": [");
                for (i, req) in required_fields.iter().enumerate() {
                    let comma = if i < required_fields.len() - 1 {
                        ","
                    } else {
                        ""
                    };
                    let _ = writeln!(output, "        \"{}\"{}", req, comma);
                }
                let _ = writeln!(output, "      ]");
            } else {
                let _ = writeln!(output, "      \"required\": []");
            }

            let _ = write!(output, "    }}");
        }
    }

    let _ = writeln!(output);
    let _ = writeln!(output, "  }}");
    let _ = writeln!(output, "}}");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A M B A S S A D O R".bold().cyan());
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

    for line in table.to_string().lines() {
        println!("   {}", line);
    }
    println!();

    Ok(())
}

fn glossa_type_to_json_schema(g_type: &GlossaType, buf: &mut String, indent: usize) {
    let ind = " ".repeat(indent);
    match g_type {
        GlossaType::Number => {
            let _ = write!(buf, "{{\n{}  \"type\": \"integer\"\n{}}}", ind, ind);
        }
        GlossaType::String => {
            let _ = write!(buf, "{{\n{}  \"type\": \"string\"\n{}}}", ind, ind);
        }
        GlossaType::Boolean => {
            let _ = write!(buf, "{{\n{}  \"type\": \"boolean\"\n{}}}", ind, ind);
        }
        GlossaType::List(inner) | GlossaType::Set(inner) => {
            let _ = write!(
                buf,
                "{{\n{}  \"type\": \"array\",\n{}  \"items\": ",
                ind, ind
            );
            let mut inner_str = String::new();
            glossa_type_to_json_schema(inner, &mut inner_str, indent + 2);
            let _ = write!(buf, "{}\n{}}}", inner_str.trim_end(), ind);
        }
        GlossaType::Map(_, _) => {
            let _ = write!(buf, "{{\n{}  \"type\": \"object\"\n{}}}", ind, ind);
        }
        GlossaType::Option(inner) => {
            glossa_type_to_json_schema(inner, buf, indent);
        }
        GlossaType::Struct { name, .. } => {
            let _ = write!(
                buf,
                "{{\n{}  \"$ref\": \"#/definitions/{}\"\n{}}}",
                ind, name, ind
            );
        }
        _ => {
            let _ = write!(
                buf,
                "{{\n{}  \"type\": [\"string\", \"number\", \"object\", \"array\", \"boolean\", \"null\"]\n{}}}",
                ind, ind
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json_schema() {
        let mut buf = String::new();
        glossa_type_to_json_schema(&GlossaType::Number, &mut buf, 0);
        assert!(buf.contains(r#""type": "integer""#));

        let mut buf = String::new();
        glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::String)), &mut buf, 0);
        assert!(buf.contains(r#""type": "array""#));
        assert!(buf.contains(r#""type": "string""#));
    }
}
