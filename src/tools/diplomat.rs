//! The Diplomat (ὁ Διπλωμάτης) - JSON Schema Generator
//!
//! This module implements the "Diplomat" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into JSON Schema representations.
use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

pub fn run_diplomat(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }
    let status = Status::start_with_symbol("Διπλωμάτης (Generating JSON Schema)", "📜");
    let source = load_source(input)?;
    let program = crate::tools::runner::analyze_source(&source)?;
    drop(status);

    let mut output = String::new();
    output.push_str(
        "{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"definitions\": {\n",
    );

    let mut first = true;
    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            if !first {
                output.push_str(",\n");
            }
            first = false;
            let _ = writeln!(output, "    \"{}\": {{", name);
            output.push_str("      \"type\": \"object\",\n      \"properties\": {\n");
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let schema = glossa_type_to_json_schema(field_type);
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(output, "        \"{}\": {}{}", field_name, schema, comma);
            }
            output.push_str("      }\n    }");
        }
    }
    output.push_str("\n  }\n}");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   D I P L O M A T".bold().magenta());
    println!("   {}", "JSON Schema".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("JSON Schema")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
    ]);
    let formatted_code = format!("```json\n{}\n```", output.trim());
    table.add_row(vec![Cell::new(formatted_code)]);
    println!("{table}\n");

    Ok(())
}

pub fn glossa_type_to_json_schema(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "{\"type\":\"integer\"}".to_string(),
        GlossaType::String => "{\"type\":\"string\"}".to_string(),
        GlossaType::Boolean => "{\"type\":\"boolean\"}".to_string(),
        GlossaType::List(inner) => format!(
            "{{\"type\":\"array\",\"items\":{}}}",
            glossa_type_to_json_schema(inner)
        ),
        GlossaType::Option(inner) => glossa_type_to_json_schema(inner),
        GlossaType::Struct { name, .. } => format!("{{\"$ref\":\"#/definitions/{}\"}}", name),
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
            "{\"type\":\"integer\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::String),
            "{\"type\":\"string\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::Boolean),
            "{\"type\":\"boolean\"}"
        );
        assert_eq!(
            glossa_type_to_json_schema(&GlossaType::List(Box::new(GlossaType::Number))),
            "{\"type\":\"array\",\"items\":{\"type\":\"integer\"}}"
        );
    }
}
