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

/// Recursively calculates the maximum loop nesting depth of a statement.
///
/// Increases depth when entering `While` or `For` loops, and explores
/// inner statements in branches (`If`, `Match`, functions).
pub fn calculate_max_depth(stmt: &AnalyzedStatement, current_depth: usize) -> usize {
    let mut max_depth = current_depth;
    match stmt {
        AnalyzedStatement::While { body, .. } | AnalyzedStatement::For { body, .. } => {
            let next_depth = current_depth + 1;
            max_depth = max_depth.max(next_depth);
            for s in body {
                max_depth = max_depth.max(calculate_max_depth(s, next_depth));
            }
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            for s in then_body {
                max_depth = max_depth.max(calculate_max_depth(s, current_depth));
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    max_depth = max_depth.max(calculate_max_depth(s, current_depth));
                }
            }
        }
        AnalyzedStatement::Match { arms, .. } => {
            for (_, stmts) in arms {
                for s in stmts {
                    max_depth = max_depth.max(calculate_max_depth(s, current_depth));
                }
            }
        }
        AnalyzedStatement::FunctionDef { body, .. }
        | AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                max_depth = max_depth.max(calculate_max_depth(s, current_depth));
            }
        }
        AnalyzedStatement::TraitDefinition { methods, .. }
        | AnalyzedStatement::TraitImplementation { methods, .. } => {
            for method in methods {
                if let Some(body) = &method.body {
                    for s in body {
                        max_depth = max_depth.max(calculate_max_depth(s, current_depth));
                    }
                }
            }
        }
        _ => {}
    }
    max_depth
}

/// Analyzes a ΓΛΩΣΣΑ source file and estimates its Big-O time complexity.
///
/// This function coordinates the parsing, semantic analysis, and AST traversal
/// using `calculate_max_depth`. The result is presented to the user in a
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

    let mut max_depth = 0;
    for stmt in &program.statements {
        max_depth = max_depth.max(calculate_max_depth(stmt, 0));
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
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt, 0), 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt, 0), 1);
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
        assert_eq!(calculate_max_depth(&outer_loop, 0), 2);
    }

    #[test]
    fn test_gnomon_calculate_max_depth_coverage() {
        // Test If and Match statements which should be covered
        let stmt = AnalyzedStatement::If {
            condition: dummy_expr(),
            then_body: vec![AnalyzedStatement::While {
                condition: dummy_expr(),
                body: vec![],
            }],
            else_body: Some(vec![AnalyzedStatement::For {
                variable: SmolStr::new("x"),
                iterator: dummy_expr(),
                body: vec![],
            }]),
        };
        assert_eq!(calculate_max_depth(&stmt, 0), 1);

        let match_stmt = AnalyzedStatement::Match {
            scrutinee: dummy_expr(),
            arms: vec![
                (*dummy_expr(), vec![AnalyzedStatement::While {
                    condition: dummy_expr(),
                    body: vec![],
                }]),
            ],
        };
        assert_eq!(calculate_max_depth(&match_stmt, 0), 1);

        let fn_stmt = AnalyzedStatement::FunctionDef {
            name: SmolStr::new("f"),
            params: vec![],
            body: vec![AnalyzedStatement::While {
                condition: dummy_expr(),
                body: vec![],
            }],
            return_type: None,
        };
        assert_eq!(calculate_max_depth(&fn_stmt, 0), 1);

        let trait_stmt = AnalyzedStatement::TraitDefinition {
            name: SmolStr::new("T"),
            methods: vec![
                crate::semantic::AnalyzedMethod {
                    name: SmolStr::new("m"),
                    params: vec![],
                    body: Some(vec![AnalyzedStatement::While {
                        condition: dummy_expr(),
                        body: vec![],
                    }]),
                    return_type: None,
                }
            ],
        };
        assert_eq!(calculate_max_depth(&trait_stmt, 0), 1);

        // Also a quick coverage of the no-op fallback
        let break_stmt = AnalyzedStatement::Break;
        assert_eq!(calculate_max_depth(&break_stmt, 0), 0);
    }
}
