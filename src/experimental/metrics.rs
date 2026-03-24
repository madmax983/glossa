//! The Metrics Tool ("Metrics")
//!
//! This module implements the "Metrics" functionality, which analyzes a ΓΛΩΣΣΑ
//! program and calculates various statistics about it.

use std::collections::HashSet;
use std::path::Path;

use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;

use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;

/// Helper function to calculate metrics from source code.
/// Returns (total_words, unique_words, total_statements).
pub fn calculate_metrics(source: &str) -> Result<(usize, usize, usize)> {
    let mut total_words = 0;
    let mut unique_words = HashSet::new();

    // Naive word counting
    for word in source.split_whitespace() {
        total_words += 1;
        unique_words.insert(word);
    }

    let ast = parse(source).map_err(|e| miette::miette!("Parse error: {}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("Semantic error: {}", e))?;

    let total_statements = program.statements.len();

    Ok((total_words, unique_words.len(), total_statements))
}

/// Run the Metrics tool on a file
pub fn run_metrics(input: &Path) -> Result<()> {
    let source = load_source(input)?;

    let status = Status::start_with_symbol("Μετρήσεις (Metrics)", "📊");

    let (total_words, unique_words, total_statements) = calculate_metrics(&source)?;

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   M E T R I C S".bold().cyan());
    println!("   {}", "Code Analysis Statistics".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        Cell::new("Metric")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
    ]);

    table.add_row(vec![
        Cell::new("Total Words"),
        Cell::new(total_words.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Unique Words"),
        Cell::new(unique_words.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Total Statements"),
        Cell::new(total_statements.to_string()),
    ]);

    let lexical_richness = if total_words > 0 {
        (unique_words as f64 / total_words as f64) * 100.0
    } else {
        0.0
    };

    table.add_row(vec![
        Cell::new("Lexical Richness"),
        Cell::new(format!("{:.1}%", lexical_richness)),
    ]);

    println!("{table}");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let (total, unique, stmts) = calculate_metrics(source).unwrap();

        // "ξ πέντε ἔστω." -> 3 words
        // "ξ λέγε." -> 2 words
        // total -> 5
        // unique -> ξ, πέντε, ἔστω., λέγε. -> 4
        // statements -> 2

        assert_eq!(total, 5);
        assert_eq!(unique, 4);
        assert_eq!(stmts, 2);
    }
}
