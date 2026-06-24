//! The Architect (ὁ Ἀρχιτέκτων) - JSON Schema Generator
//!
//! This module implements the "Architect" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into a JSON Schema.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Architect tool to generate JSON Schema from Glossa types.
pub fn run_architect(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀρχιτέκτων (Generating JSON Schema)", "🏛️");

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
    output.push_str(
        "{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"definitions\": {\n",
    );

    let mut first_type = true;
    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            if !first_type {
                output.push_str(",\n");
            }
            first_type = false;

            let _ = writeln!(output, "    \"{}\": {{", name);
            let _ = writeln!(output, "      \"type\": \"object\",");
            let _ = writeln!(output, "      \"properties\": {{");

            let mut required_fields = Vec::new();
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let (json_type, is_required) = glossa_type_to_json(field_type);
                if is_required {
                    required_fields.push(field_name.to_string());
                }
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(
                    output,
                    "        \"{}\": {{ \"type\": \"{}\" }}{}",
                    field_name, json_type, comma
                );
            }

            let _ = write!(output, "      }}");
            if !required_fields.is_empty() {
                let _ = write!(output, ",\n      \"required\": [");
                for (i, req) in required_fields.iter().enumerate() {
                    if i > 0 {
                        let _ = write!(output, ", ");
                    }
                    let _ = write!(output, "\"{}\"", req);
                }
                let _ = write!(output, "]");
            }
            let _ = write!(output, "\n    }}");
        }
    }
    output.push_str("\n  }\n}");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A R C H I T E C T".bold().cyan());
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

fn glossa_type_to_json(g_type: &GlossaType) -> (&'static str, bool) {
    match g_type {
        GlossaType::Number => ("integer", true),
        GlossaType::String => ("string", true),
        GlossaType::Boolean => ("boolean", true),
        GlossaType::List(_) => ("array", true),
        GlossaType::Set(_) => ("array", true),
        GlossaType::Map(_, _) => ("object", true),
        GlossaType::Option(inner) => (glossa_type_to_json(inner).0, false),
        _ => ("object", true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossa_type_to_json() {
        assert_eq!(glossa_type_to_json(&GlossaType::Number), ("integer", true));
        assert_eq!(glossa_type_to_json(&GlossaType::String), ("string", true));
        assert_eq!(
            glossa_type_to_json(&GlossaType::Option(Box::new(GlossaType::Number))),
            ("integer", false)
        );
    }
}
