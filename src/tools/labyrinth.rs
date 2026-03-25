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

    #[test]
    fn test_calculate_complexity_test_declaration() {
        let stmt = AnalyzedStatement::TestDeclaration {
            name: "my_test".to_string(),
            body: vec![AnalyzedStatement::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![],
                else_body: Some(vec![AnalyzedStatement::For {
                    variable: "x".into(),
                    iterator: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                        glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
                    }),
                    body: vec![],
                }]),
            }],
        };
        let complexity = calculate_complexity(&[stmt]);
        assert_eq!(complexity, 3); // 1 base + 1 if + 1 for
    }

    #[test]
    fn test_calculate_complexity_trait_implementation() {
        use crate::semantic::AnalyzedMethod;
        let stmt = AnalyzedStatement::TraitImplementation {
            trait_name: "MyTrait".into(),
            type_name: "MyType".into(),
            methods: vec![AnalyzedMethod {
                name: "my_method".into(),
                params: vec![],
                return_type: None,
                body: Some(vec![AnalyzedStatement::If {
                    condition: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::BooleanLiteral(true),
                        glossa_type: GlossaType::Boolean,
                    }),
                    then_body: vec![],
                    else_body: None,
                }]),
            }],
        };
        // The complexity visitor is normally invoked via `run_labyrinth` walking `program.statements`.
        // So we can just test `run_labyrinth` with a full source that produces this AST
        // or test the visitor function manually.
        // `calculate_complexity` doesn't natively handle `TraitImplementation` on the root level unless
        // it's called from `run_labyrinth`.
        // Let's just test `visit_statement` on it directly.
        let mut complexity = 0;
        complexity += visit_statement(&stmt);
        assert_eq!(complexity, 0); // visit_statement doesn't handle TraitImplementation directly, it's done in `run_labyrinth` iteration.
    }

    #[test]
    fn test_calculate_complexity_function_def() {
        let stmt = AnalyzedStatement::FunctionDef {
            name: "my_func".into(),
            params: vec![],
            return_type: None,
            body: vec![AnalyzedStatement::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![],
                else_body: None,
            }],
        };
        let complexity = calculate_complexity(&[stmt]);
        assert_eq!(complexity, 2); // 1 base + 1 if
    }

    #[test]
    fn test_run_labyrinth_file_not_found() {
        let result = run_labyrinth(Path::new("does_not_exist.γλ"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_labyrinth_empty_metrics() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.γλ");
        std::fs::write(&path, "ξ 1 ἔστω.").unwrap();

        let result = run_labyrinth(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_labyrinth_with_metrics() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metrics.γλ");
        std::fs::write(
            &path,
            "
            my_func ὁρίζειν ·
                «1» λέγε.
            δός.

            χαρακτήρ Τ ὁρίζειν {
                δεῖ f τῷ self.
            }.
            εἶδος Χ ὁρίζειν { χ ἀριθμοῦ. }.
            εἶδος Χ τῷ Τ ἐμπίπτειν {
                f τῷ self·
                    εἰ ἀληθές ἐστι, «1» λέγε. εἰ δὲ μή, «2» λέγε.
            }.

            δοκιμή «my test».
                εἰ ἀληθές ἐστι, «1» λέγε.
            τέλος.
        ",
        )
        .unwrap();

        let result = run_labyrinth(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_labyrinth_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("parse_error.γλ");
        std::fs::write(&path, "this is invalid code").unwrap();

        let result = run_labyrinth(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_labyrinth_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("semantic_error.γλ");
        std::fs::write(&path, "ω λέγε.").unwrap(); // Undefined variable

        let result = run_labyrinth(&path);
        assert!(result.is_err());
    }
}
