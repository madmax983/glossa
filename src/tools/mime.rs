//! The Mime (ὁ Μῖμος) - Mock JSON Generator
//!
//! This module implements the "Mime" tool, which inspects the type definitions
//! (`εἶδος`) within a ΓΛΩΣΣΑ program and translates them into mock JSON data.
//!
//! # Purpose
//!
//! Mock data generation is a common task in modern software development.
//! This tool leverages the semantic AST to automatically generate valid JSON
//! objects that match the shape of the user's defined types.

use crate::semantic::{AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

pub fn run_mime(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Μῖμος (Generating Mock Data)", "🎭");

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

    let mut mock_data = Vec::new();

    for stmt in &program.statements {
        if let AnalyzedStatement::TypeDefinition { name, fields } = stmt {
            let mut obj = String::new();
            obj.push_str("{\n");
            for (i, (field_name, field_type)) in fields.iter().enumerate() {
                let value = mock_value_for_type(field_type);
                let comma = if i < fields.len() - 1 { "," } else { "" };
                obj.push_str(&format!("  \"{}\": {}{}\n", field_name, value, comma));
            }
            obj.push('}');
            mock_data.push((name.clone(), obj));
        }
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   M I M E".bold().cyan());
    println!("   {}", "Mock JSON Data Generator".italic().dim());
    println!();

    if mock_data.is_empty() {
        println!("No type definitions found to mock.");
        return Ok(());
    }

    for (name, json) in mock_data {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);

        table.set_header(vec![
            Cell::new(format!("Mock Data for {}", name))
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

        let formatted_code = format!("```json\n{}\n```", json);
        table.add_row(vec![Cell::new(formatted_code)]);

        println!("{table}");
        println!();
    }

    Ok(())
}

fn mock_value_for_type(g_type: &GlossaType) -> String {
    match g_type {
        GlossaType::Number => "42".to_string(),
        GlossaType::String => "\"Mock String\"".to_string(),
        GlossaType::Boolean => "true".to_string(),
        _ => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_value_for_type() {
        assert_eq!(mock_value_for_type(&GlossaType::Number), "42");
        assert_eq!(mock_value_for_type(&GlossaType::String), "\"Mock String\"");
        assert_eq!(mock_value_for_type(&GlossaType::Boolean), "true");
    }
}
