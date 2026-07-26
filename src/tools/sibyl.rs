//! The Sibyl (ἡ Σίβυλλα) - Mock Data Generator
//!
//! This module implements the "Sibyl" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and generates mock JSON data based on them.
//!
//! # Purpose
//!
//! Generating realistic test data is tedious. The Sibyl acts as an oracle, reading
//! the structural definition of types and divining mock instances. This bridges
//! the gap between ancient type semantics and modern JSON tooling.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Sibyl tool to generate mock JSON from Glossa types.
pub fn run_sibyl(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Σίβυλλα (Generating Mock Data)", "🔮");

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
    output.push_str("[\n");

    let mut is_first = true;

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            if !is_first {
                output.push_str(",\n");
            }
            is_first = false;

            let _ = writeln!(output, "  {{");
            let _ = writeln!(output, "    \"_type\": \"{}\",", name);

            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let mock_val = generate_mock_value(field_type);
                let comma = if i < fields.len() - 1 { "," } else { "" };
                let _ = writeln!(output, "    \"{}\": {}{}", field_name, mock_val, comma);
            }
            output.push_str("  }");
        }
    }

    output.push_str("\n]\n");

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S I B Y L".bold().cyan());
    println!("   {}", "Mock JSON Data".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("JSON Output")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
    ]);

    let formatted_code = format!("```json\n{}\n```", output.trim());
    table.add_row(vec![Cell::new(formatted_code)]);

    println!("{table}");
    println!();

    Ok(())
}

fn generate_mock_value(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "42".to_string(),
        GlossaType::String => "\"mock_string\"".to_string(),
        GlossaType::Boolean => "true".to_string(),
        GlossaType::List(_) => "[]".to_string(),
        GlossaType::Set(_) => "[]".to_string(),
        GlossaType::Map(_, _) => "{}".to_string(),
        GlossaType::Option(inner) => generate_mock_value(inner),
        _ => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_value() {
        assert_eq!(generate_mock_value(&GlossaType::Number), "42");
        assert_eq!(generate_mock_value(&GlossaType::String), "\"mock_string\"");
        assert_eq!(generate_mock_value(&GlossaType::Boolean), "true");
        assert_eq!(generate_mock_value(&GlossaType::List(Box::new(GlossaType::Number))), "[]");
        assert_eq!(generate_mock_value(&GlossaType::Option(Box::new(GlossaType::Number))), "42");
        assert_eq!(generate_mock_value(&GlossaType::Unknown), "null");
    }
}
