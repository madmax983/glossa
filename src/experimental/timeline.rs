//! The Timeline Tool ("Chronos")
//!
//! This module implements an experimental feature that visualizes
//! the control flow and variable lifecycle of a ΓΛΩΣΣΑ program.
//! It acts as a static analysis simulator showing how variables
//! are created, modified, and used.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};

pub fn generate_timeline(program: &AnalyzedProgram) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL).set_header(vec![
        Cell::new("Step")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Event")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new("Variables Affected")
            .add_attribute(Attribute::Bold)
            .fg(Color::Green),
    ]);

    let mut step_count = 1;
    for stmt in &program.statements {
        analyze_step(&mut table, stmt, &mut step_count);
    }

    format!(
        "\n   \x1b[38;5;14m\x1b[1mΓ Λ Ω Σ Σ Α   T I M E L I N E\x1b[0m\n   \x1b[2m\x1b[3mChronos Control Flow Simulator\x1b[0m\n\n{}",
        table
    )
}

fn analyze_step(table: &mut Table, stmt: &AnalyzedStatement, step: &mut usize) {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value: _,
            mutable,
        } => {
            let mutability = if *mutable { "Mutable" } else { "Immutable" };
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new(format!("Birth (let {})", name)),
                Cell::new(format!("+ {} [{}]", name, mutability)),
            ]);
            *step += 1;
        }
        AnalyzedStatement::Assignment { name, value: _ } => {
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new(format!("Mutation ({})", name)),
                Cell::new(format!("~ {} modified", name)),
            ]);
            *step += 1;
        }
        AnalyzedStatement::If {
            condition: _,
            then_body,
            else_body,
        } => {
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new("Branch (If)"),
                Cell::new("Control Flow Diverges"),
            ]);
            *step += 1;

            // Just simulate the block boundaries, we don't do full static analysis
            table.add_row(vec![
                Cell::new("".to_string()),
                Cell::new("↳ Then Block Start".to_string()),
                Cell::new("".to_string()),
            ]);

            for b_stmt in then_body {
                analyze_step(table, b_stmt, step);
            }

            if let Some(e_body) = else_body {
                table.add_row(vec![
                    Cell::new("".to_string()),
                    Cell::new("↳ Else Block Start".to_string()),
                    Cell::new("".to_string()),
                ]);
                for b_stmt in e_body {
                    analyze_step(table, b_stmt, step);
                }
            }
        }
        AnalyzedStatement::While { condition: _, body } => {
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new("Loop (While)"),
                Cell::new("Control Flow Cycles"),
            ]);
            *step += 1;
            table.add_row(vec![
                Cell::new("".to_string()),
                Cell::new("↻ Loop Body Start".to_string()),
                Cell::new("".to_string()),
            ]);
            for b_stmt in body {
                analyze_step(table, b_stmt, step);
            }
        }
        AnalyzedStatement::For {
            variable,
            iterator: _,
            body,
        } => {
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new(format!("Iteration (For {})", variable)),
                Cell::new(format!("+ {} [Loop Binding]", variable)),
            ]);
            *step += 1;
            table.add_row(vec![
                Cell::new("".to_string()),
                Cell::new("↻ Loop Body Start".to_string()),
                Cell::new("".to_string()),
            ]);
            for b_stmt in body {
                analyze_step(table, b_stmt, step);
            }
        }
        AnalyzedStatement::Print(_) | AnalyzedStatement::Query(_) => {
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new("I/O (Print)"),
                Cell::new("Side Effect"),
            ]);
            *step += 1;
        }
        _ => {
            // Other statements
            table.add_row(vec![
                Cell::new(step.to_string()),
                Cell::new("Action"),
                Cell::new("---"),
            ]);
            *step += 1;
        }
    }
}

use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::runner::load_source;
use miette::Result;
use std::path::Path;

pub fn run_timeline(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let source = load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let analyzed = analyze_program(&ast)?;

    let timeline = generate_timeline(&analyzed);
    println!("{}", timeline);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_generation() {
        let source = "μετά ξ 5 ἔστω. ξ 10 γίγνεται. εἰ ξ 5 μεῖζον ᾖ, «μεῖζον» λέγε.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let timeline = generate_timeline(&program);

        assert!(timeline.contains("Birth (let ξ)"));
        assert!(timeline.contains("Mutation (ξ)"));
        assert!(timeline.contains("Branch (If)"));
    }
}
