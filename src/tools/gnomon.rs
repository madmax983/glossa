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

/// Analyzes a statement to calculate its maximum loop depth.
///
/// Just as a gnomon casts a shadow to indicate time, this function casts a shadow
/// over the structure of a program to estimate its execution time complexity.
/// It tracks the maximum nesting depth of `while` and `for` loops.
pub fn calculate_max_depth(stmt: &AnalyzedStatement) -> usize {
    match stmt {
        AnalyzedStatement::While { body, .. } | AnalyzedStatement::For { body, .. } => {
            let mut max_child = 0;
            for s in body {
                let d = calculate_max_depth(s);
                if d > max_child {
                    max_child = d;
                }
            }
            1 + max_child
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            let mut max_child = 0;
            for s in then_body {
                let d = calculate_max_depth(s);
                if d > max_child {
                    max_child = d;
                }
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    let d = calculate_max_depth(s);
                    if d > max_child {
                        max_child = d;
                    }
                }
            }
            max_child
        }
        AnalyzedStatement::Match { arms, .. } => {
            let mut max_child = 0;
            for (_, stmts) in arms {
                for s in stmts {
                    let d = calculate_max_depth(s);
                    if d > max_child {
                        max_child = d;
                    }
                }
            }
            max_child
        }
        AnalyzedStatement::FunctionDef { body, .. }
        | AnalyzedStatement::TestDeclaration { body, .. } => {
            let mut max_child = 0;
            for s in body {
                let d = calculate_max_depth(s);
                if d > max_child {
                    max_child = d;
                }
            }
            max_child
        }
        _ => 0,
    }
}

/// Analyzes a ΓΛΩΣΣΑ source file and estimates its Big-O time complexity.
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

    let mut max_depth = 0;
    for stmt in &program.statements {
        let d = calculate_max_depth(stmt);
        if d > max_depth {
            max_depth = d;
        }
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
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
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
        assert_eq!(calculate_max_depth(&outer_loop), 2);
    }

    #[test]
    fn test_gnomon_if_statement() {
        let inner_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        let stmt = AnalyzedStatement::If {
            condition: dummy_expr(),
            then_body: vec![inner_loop],
            else_body: Some(vec![]),
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_match_statement() {
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let stmt = AnalyzedStatement::Match {
            expression: dummy_expr(),
            arms: vec![(dummy_expr(), vec![inner_loop])],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_function_def() {
        let inner_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        let stmt = AnalyzedStatement::FunctionDef {
            name: SmolStr::new("test"),
            params: vec![],
            return_type: GlossaType::Void,
            body: vec![inner_loop],
            is_public: false,
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_test_declaration() {
        let inner_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        let stmt = AnalyzedStatement::TestDeclaration {
            name: SmolStr::new("test"),
            body: vec![inner_loop],
        };
        assert_eq!(calculate_max_depth(&stmt), 1);
    }

    #[test]
    fn test_gnomon_other_statement() {
        let stmt = AnalyzedStatement::Expression(dummy_expr());
        assert_eq!(calculate_max_depth(&stmt), 0);
    }
}
