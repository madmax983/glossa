//! The Gnomon (ὁ Γνώμων) - Big-O Complexity Estimator
//!
//! This module implements the "Gnomon" tool, which estimates the Big-O time complexity
//! of a ΓΛΩΣΣΑ program by statically analyzing loop depth in the semantic AST.
//!
//! # Purpose
//!
//! A gnomon is the part of a sundial that casts a shadow, used to indicate the time.
//! This tool casts a shadow over the program's AST to estimate its execution time complexity.
use std::path::Path;

use crate::semantic::AnalyzedStatement;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
/// Calculates the maximum loop depth by traversing the Abstract Syntax Tree.
///
/// Just as a gnomon casts a shadow to indicate time, this function casts a shadow
/// over the structure of a program to estimate its execution time complexity.
/// It tracks the maximum nesting depth of `while` and `for` loops.
pub fn calculate_loop_depth(statements: &[AnalyzedStatement], current_depth: usize) -> usize {
    let mut max_depth = current_depth;

    for stmt in statements {
        let depth = match stmt {
            AnalyzedStatement::While { body, .. } => {
                let inner = calculate_loop_depth(body, current_depth + 1);
                std::cmp::max(current_depth + 1, inner)
            }
            AnalyzedStatement::For { body, .. } => {
                let inner = calculate_loop_depth(body, current_depth + 1);
                std::cmp::max(current_depth + 1, inner)
            }
            AnalyzedStatement::If {
                then_body,
                else_body,
                ..
            } => {
                let mut d = calculate_loop_depth(then_body, current_depth);
                if let Some(else_stmts) = else_body {
                    d = std::cmp::max(d, calculate_loop_depth(else_stmts, current_depth));
                }
                d
            }
            AnalyzedStatement::Match { arms, .. } => {
                let mut d = current_depth;
                for (_, stmts) in arms {
                    d = std::cmp::max(d, calculate_loop_depth(stmts, current_depth));
                }
                d
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                calculate_loop_depth(body, current_depth)
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                calculate_loop_depth(body, current_depth)
            }
            AnalyzedStatement::TraitDefinition { methods, .. } => {
                let mut d = current_depth;
                for m in methods {
                    if let Some(body) = &m.body {
                        d = std::cmp::max(d, calculate_loop_depth(body, current_depth));
                    }
                }
                d
            }
            AnalyzedStatement::TraitImplementation { methods, .. } => {
                let mut d = current_depth;
                for m in methods {
                    if let Some(body) = &m.body {
                        d = std::cmp::max(d, calculate_loop_depth(body, current_depth));
                    }
                }
                d
            }
            _ => current_depth,
        };
        max_depth = std::cmp::max(max_depth, depth);
    }

    max_depth
}
///
/// This function coordinates the parsing, semantic analysis, and AST traversal
/// using the [`GnomonVisitor`]. The result is presented to the user in a
/// stylized terminal table.
///
/// # Errors
///
/// Returns a [`miette::Result`] if:
/// - The specified file cannot be found.
/// - The source file contains syntax or semantic errors.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::gnomon::run_gnomon;
/// use std::path::Path;
///
/// let input = Path::new("algorithm.γλ");
/// if let Err(e) = run_gnomon(&input) {
///     eprintln!("Failed to estimate complexity: {}", e);
/// }
/// ```
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

    let max_depth = calculate_loop_depth(&program.statements, 0);

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
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        let max_depth = calculate_loop_depth(&[stmt], 0);
        assert_eq!(max_depth, 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let max_depth = calculate_loop_depth(&[stmt], 0);
        assert_eq!(max_depth, 1);
    }

    #[test]
    fn test_gnomon_nested_loops() {
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("y"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let outer_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![inner_loop],
        };
        let max_depth = calculate_loop_depth(&[outer_loop], 0);
        assert_eq!(max_depth, 2);
    }
}
