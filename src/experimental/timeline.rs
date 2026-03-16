//! The Chronos Tool ("Timeline")
//!
//! This module implements the "Chronos" functionality, which tracks the lifetime
//! and mutability lifecycle of variables over time.

use crate::parser::parse;
use crate::semantic::{AnalyzedStatement, analyze_program};
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Run the Timeline tool on a file
///
/// Reads the source file, parses it, analyzes it, and prints the timeline table to stdout.
pub fn run_timeline(input_path: &Path) -> Result<()> {
    let mut status = Status::start("Reading file...");
    let source = std::fs::read_to_string(input_path).into_diagnostic()?;

    status.update("Parsing and analyzing...");
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    status.success();

    println!("\n{}\n", "⏳ Timeline of Variable Lifecycle".bold().cyan());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Event Type").add_attribute(Attribute::Bold),
            Cell::new("Variable(s)").add_attribute(Attribute::Bold),
            Cell::new("Description").add_attribute(Attribute::Bold),
        ]);

    for stmt in &program.statements {
        add_statement_to_timeline(&mut table, stmt, 0);
    }

    if table.row_iter().count() == 0 {
        table.add_row(vec![
            Cell::new("None").fg(Color::DarkGrey),
            Cell::new(""),
            Cell::new("No timeline events found."),
        ]);
    }

    println!("{table}");

    Ok(())
}

fn add_statement_to_timeline(table: &mut Table, stmt: &AnalyzedStatement, depth: usize) {
    let indent = "  ".repeat(depth);

    match stmt {
        AnalyzedStatement::Binding { name, mutable, .. } => {
            let desc = format!("{indent}Variable was born.");
            let color = if *mutable {
                Color::Yellow
            } else {
                Color::Green
            };
            let event = if *mutable { "Birth (Mut)" } else { "Birth" };
            table.add_row(vec![
                Cell::new(event).fg(color),
                Cell::new(name.as_str()),
                Cell::new(desc),
            ]);
        }
        AnalyzedStatement::Assignment { name, .. } => {
            let desc = format!("{indent}Variable was mutated.");
            table.add_row(vec![
                Cell::new("Mutation").fg(Color::Red),
                Cell::new(name.as_str()),
                Cell::new(desc),
            ]);
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            let desc = format!("{indent}Control flow diverged.");
            table.add_row(vec![
                Cell::new("Divergence").fg(Color::Magenta),
                Cell::new(""),
                Cell::new(desc),
            ]);
            for b in then_body {
                add_statement_to_timeline(table, b, depth + 1);
            }
            if let Some(eb) = else_body {
                for b in eb {
                    add_statement_to_timeline(table, b, depth + 1);
                }
            }
        }
        AnalyzedStatement::While { body, .. } => {
            let desc = format!("{indent}Loop boundary reached.");
            table.add_row(vec![
                Cell::new("Divergence").fg(Color::Magenta),
                Cell::new(""),
                Cell::new(desc),
            ]);
            for b in body {
                add_statement_to_timeline(table, b, depth + 1);
            }
        }
        AnalyzedStatement::For { variable, body, .. } => {
            let desc = format!("{indent}Variable was born via loop.");
            table.add_row(vec![
                Cell::new("Birth (Mut)").fg(Color::Yellow),
                Cell::new(variable.as_str()),
                Cell::new(desc.clone()),
            ]);

            table.add_row(vec![
                Cell::new("Divergence").fg(Color::Magenta),
                Cell::new(""),
                Cell::new(format!("{indent}Loop boundary reached.")),
            ]);
            for b in body {
                add_statement_to_timeline(table, b, depth + 1);
            }
        }
        AnalyzedStatement::Match { arms, .. } => {
            let desc = format!("{indent}Control flow matched.");
            table.add_row(vec![
                Cell::new("Divergence").fg(Color::Magenta),
                Cell::new(""),
                Cell::new(desc),
            ]);
            for (_, body) in arms {
                for b in body {
                    add_statement_to_timeline(table, b, depth + 1);
                }
            }
        }
        AnalyzedStatement::Print(_) | AnalyzedStatement::Query(_) => {
            let desc = format!("{indent}IO side-effect occurred.");
            table.add_row(vec![
                Cell::new("Side-Effect").fg(Color::Cyan),
                Cell::new(""),
                Cell::new(desc),
            ]);
        }
        AnalyzedStatement::FunctionDef { body, .. } => {
            let desc = format!("{indent}Function body defined.");
            table.add_row(vec![
                Cell::new("Scope Open").fg(Color::DarkGrey),
                Cell::new(""),
                Cell::new(desc),
            ]);
            for b in body {
                add_statement_to_timeline(table, b, depth + 1);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_statement_to_timeline() {
        // Quick integration test: parse and analyze a simple code to get a timeline.
        let source = "
            μετά ξ πέντε ἔστω.
            ξ δέκα γίγνεται.
            «χαῖρε» λέγε.
        ";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        for stmt in &program.statements {
            add_statement_to_timeline(&mut table, stmt, 0);
        }

        let table_str = format!("{table}");

        // Assert we have our table entries.
        assert!(table_str.contains("Birth"));
        assert!(table_str.contains("ξ")); // variable name should be inside the table output
        assert!(table_str.contains("Mutation"));
        assert!(table_str.contains("Side-Effect"));
    }
}
