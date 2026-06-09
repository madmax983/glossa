//! The Gnomon (ὁ Γνώμων) - Big-O Complexity Estimator
//!
//! This module implements the "Gnomon" tool, which estimates the Big-O time complexity
//! of a ΓΛΩΣΣΑ program by statically analyzing loop depth in the semantic AST.
//!
//! # Purpose
//!
//! A gnomon is the part of a sundial that casts a shadow, used to indicate the time.
//! This tool casts a shadow over the program's AST to estimate its execution time complexity.

use crate::semantic::AnalyzedStatement;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// A visitor that traverses the Abstract Syntax Tree to calculate loop depth.
///
/// Just as a gnomon casts a shadow to indicate time, this visitor casts a shadow
/// over the structure of a program to estimate its execution time complexity.
/// It tracks the maximum nesting depth of `while` and `for` loops.
/// Recursively visits a statement and updates loop depth metrics.
pub fn visit_statement(stmt: &AnalyzedStatement, current_depth: &mut usize, max_depth: &mut usize) {
    match stmt {
        AnalyzedStatement::While { body, .. } => {
            *current_depth += 1;
            if *current_depth > *max_depth {
                *max_depth = *current_depth;
            }
            for s in body {
                visit_statement(s, current_depth, max_depth);
            }
            *current_depth -= 1;
        }
        AnalyzedStatement::For { body, .. } => {
            *current_depth += 1;
            if *current_depth > *max_depth {
                *max_depth = *current_depth;
            }
            for s in body {
                visit_statement(s, current_depth, max_depth);
            }
            *current_depth -= 1;
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            for s in then_body {
                visit_statement(s, current_depth, max_depth);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    visit_statement(s, current_depth, max_depth);
                }
            }
        }
        AnalyzedStatement::Match { arms, .. } => {
            for (_, stmts) in arms {
                for s in stmts {
                    visit_statement(s, current_depth, max_depth);
                }
            }
        }
        AnalyzedStatement::FunctionDef { body, .. } => {
            for s in body {
                visit_statement(s, current_depth, max_depth);
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                visit_statement(s, current_depth, max_depth);
            }
        }
        _ => {}
    }
}

pub fn run_gnomon(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Γνώμων (Estimating Complexity)", "⏳");

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
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    status.success();

    let mut current_depth = 0;
    let mut max_depth = 0;
    for stmt in &program.statements {
        visit_statement(stmt, &mut current_depth, &mut max_depth);
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   G N O M O N".cyan().bold());
    println!(
        "   {}",
        format!("Complexity Estimate for {}", input.display())
            .italic()
            .dim()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Metric")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);

    let complexity = if max_depth == 0 {
        "O(1)".to_string()
    } else if max_depth == 1 {
        "O(N)".to_string()
    } else {
        format!("O(N^{})", max_depth)
    };

    table.add_row(vec![
        Cell::new("Max Loop Depth"),
        Cell::new(max_depth.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Estimated Big-O"),
        Cell::new(complexity).fg(if max_depth > 2 {
            Color::Red
        } else if max_depth == 2 {
            Color::Yellow
        } else {
            Color::Green
        }),
    ]);

    println!("{table}");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
    use smol_str::SmolStr;

    fn dummy_expr() -> Box<AnalyzedExpr> {
        Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        })
    }

    #[test]
    fn test_gnomon_while_loop() {
        let mut current_depth = 0;
        let mut max_depth = 0;
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        visit_statement(&stmt, &mut current_depth, &mut max_depth);
        assert_eq!(max_depth, 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let mut current_depth = 0;
        let mut max_depth = 0;
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        visit_statement(&stmt, &mut current_depth, &mut max_depth);
        assert_eq!(max_depth, 1);
    }

    #[test]
    fn test_gnomon_nested_loops() {
        let mut current_depth = 0;
        let mut max_depth = 0;
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("y"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let outer_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![inner_loop],
        };
        visit_statement(&outer_loop, &mut current_depth, &mut max_depth);
        assert_eq!(max_depth, 2);
    }
}
