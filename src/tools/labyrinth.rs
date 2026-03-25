//! The Labyrinth Tool ("Labyrinth")
//!
//! This module implements a cyclomatic complexity analyzer for ΓΛΩΣΣΑ programs.

use crate::parser::parse;
use crate::semantic::{AnalyzedStatement, analyze_program};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Labyrinth tool to analyze cyclomatic complexity
pub fn run_labyrinth(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀνάλυσις Λαβυρίνθου (Analyzing Maze)", "🦬");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error(e.to_string());
            return Err(e);
        }
    };

    let ast = match parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            status.error(e.to_string());
            return Err(miette::miette!("{}", e));
        }
    };

    let program = match analyze_program(&ast) {
        Ok(prog) => prog,
        Err(e) => {
            status.error(e.to_string());
            return Err(miette::miette!("{}", e));
        }
    };

    let mut metrics = Vec::new();

    for stmt in &program.statements {
        match stmt {
            AnalyzedStatement::FunctionDef { name, body, .. } => {
                let complexity = calculate_complexity(body);
                metrics.push((name.to_string(), "Function".to_string(), complexity));
            }
            AnalyzedStatement::TraitImplementation {
                methods,
                type_name,
                trait_name,
                ..
            } => {
                for method in methods {
                    if let Some(body) = &method.body {
                        let complexity = calculate_complexity(body);
                        let name = format!("{} as {}::{}", type_name, trait_name, method.name);
                        metrics.push((name, "Method".to_string(), complexity));
                    }
                }
            }
            AnalyzedStatement::TestDeclaration { name, body } => {
                let complexity = calculate_complexity(body);
                metrics.push((name.to_string(), "Test".to_string(), complexity));
            }
            _ => {} // Top-level statements that aren't definitions don't get their own metric row for now, but we could add a "global" scope.
        }
    }

    status.success();

    if metrics.is_empty() {
        println!();
        println!(
            "   {}",
            "Λ Α Β Υ Ρ Ι Ν Θ Ο Σ   (L A B Y R I N T H)".bold().cyan()
        );
        println!(
            "   {}",
            "No functions or methods found to analyze.".italic().dim()
        );
        println!();
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Entity", "Type", "Complexity", "Risk"]);

    let mut total_complexity = 0;

    for (name, entity_type, complexity) in metrics {
        total_complexity += complexity;
        let risk = match complexity {
            1..=5 => Cell::new("Green (Simple)").fg(Color::Green),
            6..=10 => Cell::new("Yellow (Moderate)").fg(Color::Yellow),
            _ => Cell::new("Red (Labyrinth)").fg(Color::Red),
        };
        table.add_row(vec![
            Cell::new(name).fg(Color::Cyan),
            Cell::new(entity_type),
            Cell::new(complexity.to_string()),
            risk,
        ]);
    }

    println!();
    println!(
        "   {}",
        "Λ Α Β Υ Ρ Ι Ν Θ Ο Σ   (L A B Y R I N T H)".bold().cyan()
    );
    println!("   {}", "Cyclomatic Complexity Analysis".italic().dim());
    println!();
    println!("{}", table);
    println!();
    println!("   Total Complexity: {}", total_complexity);
    println!();

    Ok(())
}

fn calculate_complexity(statements: &[AnalyzedStatement]) -> usize {
    // Base complexity is 1 for the function/block itself.
    let mut complexity = 1;

    for stmt in statements {
        complexity += visit_statement(stmt);
    }

    complexity
}

fn visit_statement(stmt: &AnalyzedStatement) -> usize {
    match stmt {
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            let mut c = 1; // 1 for the if branch
            c += calculate_complexity_no_base(then_body);
            if let Some(else_b) = else_body {
                c += calculate_complexity_no_base(else_b);
            }
            c
        }
        AnalyzedStatement::While { body, .. } => {
            let mut c = 1; // 1 for the loop
            c += calculate_complexity_no_base(body);
            c
        }
        AnalyzedStatement::For { body, .. } => {
            let mut c = 1; // 1 for the loop
            c += calculate_complexity_no_base(body);
            c
        }
        AnalyzedStatement::Match { arms, .. } => {
            let mut c = arms.len().saturating_sub(1); // N branches = N-1 decisions
            for (_, body) in arms {
                c += calculate_complexity_no_base(body);
            }
            c
        }
        AnalyzedStatement::FunctionDef { body, .. } => {
            // If there's a nested function, we usually analyze it separately,
            // but for a strict AST walk, we could add its complexity.
            // Let's just walk it.
            calculate_complexity_no_base(body)
        }
        AnalyzedStatement::TestDeclaration { body, .. } => calculate_complexity_no_base(body),
        _ => 0, // Other statements don't branch
    }
}

fn calculate_complexity_no_base(statements: &[AnalyzedStatement]) -> usize {
    let mut complexity = 0;
    for stmt in statements {
        complexity += visit_statement(stmt);
    }
    complexity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

    #[test]
    fn test_calculate_complexity() {
        // Base complexity is 1
        // 1 if statement (+1)
        // 1 while statement (+1)
        // 1 for statement (+1)
        // 1 match statement with 3 arms (+2 decisions)
        let stmts = vec![
            AnalyzedStatement::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![],
                else_body: None,
            },
            AnalyzedStatement::While {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                body: vec![],
            },
            AnalyzedStatement::For {
                variable: "x".into(),
                iterator: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                    glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
                }),
                body: vec![],
            },
            AnalyzedStatement::Match {
                scrutinee: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                arms: vec![
                    (
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        },
                        vec![],
                    ),
                    (
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(2),
                            glossa_type: GlossaType::Number,
                        },
                        vec![],
                    ),
                    (
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(3),
                            glossa_type: GlossaType::Number,
                        },
                        vec![],
                    ),
                ],
            },
        ];

        // Total complexity should be:
        // 1 (base) + 1 (if) + 1 (while) + 1 (for) + 2 (match branches) = 6
        let complexity = calculate_complexity(&stmts);
        assert_eq!(complexity, 6);
    }
}
