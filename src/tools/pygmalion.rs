//! The Pygmalion (ὁ Πυγμαλίων) - JSON Schema Generator
//!
//! This module implements the "Pygmalion" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema.
//!
//! # Purpose
//!
//! Pygmalion fell in love with his statue, and it came to life. This tool takes
//! abstract Glossa types and breathes life into them as standard JSON Schemas,
//! allowing Glossa to act as a language-agnostic data definition language.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

pub fn run_pygmalion(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Πυγμαλίων (Generating JSON Schema)", "🗿");

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

    let mut types = Vec::new();
    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            types.push((name, fields));
        }
    }

    for (i, (name, fields)) in types.iter().enumerate() {
        let _ = writeln!(output, "    \"{}\": {{", name);
        let _ = writeln!(output, "      \"type\": \"object\",");
        let _ = writeln!(output, "      \"properties\": {{");

        for (j, (field_name, field_type)) in fields.iter().enumerate() {
            let json_type = glossa_type_to_json_schema(field_type);
            let comma = if j < fields.len() - 1 { "," } else { "" };
            let _ = writeln!(output, "        \"{}\": {}{}", field_name, json_type, comma);
        }

        let _ = writeln!(output, "      }}");
        let comma = if i < types.len() - 1 { "," } else { "" };
        let _ = writeln!(output, "    }}{}", comma);
    }

    let _ = writeln!(output, "  }}");
    let _ = writeln!(output, "}}");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   P Y G M A L I O N".bold().cyan());
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

fn glossa_type_to_json_schema(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "{\"type\": \"integer\"}".to_string(),
        GlossaType::String => "{\"type\": \"string\"}".to_string(),
        GlossaType::Boolean => "{\"type\": \"boolean\"}".to_string(),
        GlossaType::List(inner) => format!(
            "{{\"type\": \"array\", \"items\": {}}}",
            glossa_type_to_json_schema(inner)
        ),
        GlossaType::Set(inner) => format!(
            "{{\"type\": \"array\", \"items\": {}}}",
            glossa_type_to_json_schema(inner)
        ),
        GlossaType::Map(_, _) => "{\"type\": \"object\"}".to_string(),
        GlossaType::Option(inner) => glossa_type_to_json_schema(inner), // JSON schema required array handles optional
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
            "{\"type\": \"array\", \"items\": {\"type\": \"integer\"}}"
        );
        assert_eq!(glossa_type_to_json_schema(&GlossaType::Unknown), "{}");
    }
}
