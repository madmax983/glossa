//! The Timeline Tool (ὁ Χρόνος)
//!
//! An experimental static analysis simulator that maps out events such as
//! variable birth, mutation, divergence of control flow, and IO side-effects
//! for a given Glossa program.

use comfy_table::{Cell, Color, Table};
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::parser::parse;
use crate::semantic::analyzer::analyze_program;
use crate::semantic::model::{AnalyzedExprKind, AnalyzedStatement};

/// Runs the Timeline tool on a given Glossa source file.
pub fn run_timeline(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path).into_diagnostic()?;
    let mut buffer = Vec::new();
    run_timeline_inner(&source, &mut buffer)?;
    print!("{}", String::from_utf8_lossy(&buffer));
    Ok(())
}

fn run_timeline_inner(source: &str, out: &mut impl std::io::Write) -> Result<()> {
    let ast = parse(source)?;
    let program = analyze_program(&ast)?;

    let mut table = Table::new();
    table.set_header(vec!["Time (Tick)", "Event Type", "Target", "Details"]);

    let mut tick = 0;
    for stmt in &program.statements {
        analyze_statement_timeline(stmt, &mut table, &mut tick);
    }

    writeln!(out, "\n=== THE TIMELINE (ὁ Χρόνος) ===\n").into_diagnostic()?;
    writeln!(out, "{}", table).into_diagnostic()?;

    Ok(())
}

fn analyze_statement_timeline(stmt: &AnalyzedStatement, table: &mut Table, tick: &mut usize) {
    *tick += 1;
    let current_tick = *tick;

    match stmt {
        AnalyzedStatement::Binding { name, mutable, .. } => {
            let mutability = if *mutable { "Mutable" } else { "Immutable" };
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Birth").fg(Color::Green),
                Cell::new(name.to_string()),
                Cell::new(format!("Created as {}", mutability)),
            ]);
        }
        AnalyzedStatement::Assignment { name, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Mutation").fg(Color::Yellow),
                Cell::new(name.to_string()),
                Cell::new("Value updated"),
            ]);
        }
        AnalyzedStatement::Print(exprs) => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Side Effect").fg(Color::Cyan),
                Cell::new("stdout"),
                Cell::new(format!("Prints {} items", exprs.len())),
            ]);
        }
        AnalyzedStatement::Query(exprs) => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Side Effect").fg(Color::Cyan),
                Cell::new("stdout"),
                Cell::new(format!("Queries {} items", exprs.len())),
            ]);
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Divergence").fg(Color::Magenta),
                Cell::new("Control Flow"),
                Cell::new("Branching occurs (If)"),
            ]);
            for sub_stmt in then_body {
                analyze_statement_timeline(sub_stmt, table, tick);
            }
            if let Some(else_stmts) = else_body {
                for sub_stmt in else_stmts {
                    analyze_statement_timeline(sub_stmt, table, tick);
                }
            }
            *tick += 1;
            table.add_row(vec![
                Cell::new(tick.to_string()),
                Cell::new("Convergence").fg(Color::Magenta),
                Cell::new("Control Flow"),
                Cell::new("Branches merge"),
            ]);
        }
        AnalyzedStatement::While { body, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Loop Start").fg(Color::Blue),
                Cell::new("Control Flow"),
                Cell::new("While loop begins"),
            ]);
            for sub_stmt in body {
                analyze_statement_timeline(sub_stmt, table, tick);
            }
            *tick += 1;
            table.add_row(vec![
                Cell::new(tick.to_string()),
                Cell::new("Loop End").fg(Color::Blue),
                Cell::new("Control Flow"),
                Cell::new("While loop evaluates condition"),
            ]);
        }
        AnalyzedStatement::For { variable, body, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Loop Start").fg(Color::Blue),
                Cell::new(variable.to_string()),
                Cell::new("For loop iterates"),
            ]);
            for sub_stmt in body {
                analyze_statement_timeline(sub_stmt, table, tick);
            }
            *tick += 1;
            table.add_row(vec![
                Cell::new(tick.to_string()),
                Cell::new("Loop End").fg(Color::Blue),
                Cell::new("Control Flow"),
                Cell::new("For loop ends"),
            ]);
        }
        AnalyzedStatement::Match { arms, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Divergence").fg(Color::Magenta),
                Cell::new("Control Flow"),
                Cell::new(format!("Match expression with {} arms", arms.len())),
            ]);
            for (_, body) in arms {
                for sub_stmt in body {
                    analyze_statement_timeline(sub_stmt, table, tick);
                }
            }
            *tick += 1;
            table.add_row(vec![
                Cell::new(tick.to_string()),
                Cell::new("Convergence").fg(Color::Magenta),
                Cell::new("Control Flow"),
                Cell::new("Match branches merge"),
            ]);
        }
        AnalyzedStatement::Expression(exprs) => {
            for expr in exprs {
                if let AnalyzedExprKind::MethodCall { method, .. } = &expr.expr {
                    table.add_row(vec![
                        Cell::new(current_tick.to_string()),
                        Cell::new("Call").fg(Color::DarkYellow),
                        Cell::new(method.to_string()),
                        Cell::new("Method call"),
                    ]);
                } else if let AnalyzedExprKind::FunctionCall { func, .. } = &expr.expr {
                    table.add_row(vec![
                        Cell::new(current_tick.to_string()),
                        Cell::new("Call").fg(Color::DarkYellow),
                        Cell::new(func.to_string()),
                        Cell::new("Function call"),
                    ]);
                }
            }
        }
        AnalyzedStatement::FunctionDef { name, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Definition").fg(Color::White),
                Cell::new(name.to_string()),
                Cell::new("Function defined"),
            ]);
        }
        AnalyzedStatement::TypeDefinition { name, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Definition").fg(Color::White),
                Cell::new(name.to_string()),
                Cell::new("Type (Struct) defined"),
            ]);
        }
        AnalyzedStatement::TraitDefinition { name, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Definition").fg(Color::White),
                Cell::new(name.to_string()),
                Cell::new("Trait defined"),
            ]);
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            ..
        } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Implementation").fg(Color::White),
                Cell::new(format!("{} for {}", trait_name, type_name)),
                Cell::new("Trait implemented"),
            ]);
        }
        AnalyzedStatement::Return { .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Exit").fg(Color::Red),
                Cell::new("Control Flow"),
                Cell::new("Return from function"),
            ]);
        }
        AnalyzedStatement::Break => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Exit").fg(Color::Red),
                Cell::new("Control Flow"),
                Cell::new("Break from loop"),
            ]);
        }
        AnalyzedStatement::Continue => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Jump").fg(Color::Yellow),
                Cell::new("Control Flow"),
                Cell::new("Continue loop"),
            ]);
        }
        AnalyzedStatement::TestDeclaration { name, .. } => {
            table.add_row(vec![
                Cell::new(current_tick.to_string()),
                Cell::new("Definition").fg(Color::White),
                Cell::new(name.to_string()),
                Cell::new("Test declared"),
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_output() {
        let source = "
            ξ μετά πέντε ἔστω.
            «χαῖρε» λέγε.
            ξ 10 γίγνεται.
        ";
        let mut buffer = Vec::new();
        run_timeline_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("THE TIMELINE"));
        assert!(output.contains("Birth"));
        assert!(output.contains("Mutation"));
        assert!(output.contains("Side Effect"));
        assert!(output.contains("ξ"));
    }
}
