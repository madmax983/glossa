use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::parser::parse;
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, analyze_program};

pub fn run_timeline(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path).into_diagnostic()?;
    let ast = parse(&source)?;
    let program = analyze_program(&ast)?;

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   Χ Ρ Ο Ν Ο Σ".bold().cyan());
    println!(
        "   {}",
        "The Timeline (Static Analysis Simulator)".italic().dim()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Event")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Target").add_attribute(Attribute::Bold),
        Cell::new("Details").add_attribute(Attribute::Bold),
        Cell::new("Type").add_attribute(Attribute::Bold),
    ]);

    traverse_statements(&mut table, &program.statements, 0);

    println!("{}", table);

    Ok(())
}

fn traverse_statements(table: &mut Table, statements: &[AnalyzedStatement], depth: usize) {
    let prefix = "  ".repeat(depth);
    for stmt in statements {
        match stmt {
            AnalyzedStatement::Binding {
                name,
                value,
                mutable,
            } => {
                let type_str = format!("{:?}", value.glossa_type);
                let mut_str = if *mutable { " (Mut)" } else { "" };
                table.add_row(vec![
                    Cell::new(format!("{}Birth", prefix)).fg(Color::Green),
                    Cell::new(name.as_str()),
                    Cell::new(format!("Initial value assigned{}", mut_str)),
                    Cell::new(type_str).fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::Assignment { name, value } => {
                let type_str = format!("{:?}", value.glossa_type);
                table.add_row(vec![
                    Cell::new(format!("{}Mutation", prefix)).fg(Color::Red),
                    Cell::new(name.as_str()),
                    Cell::new("Value reassigned"),
                    Cell::new(type_str).fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::Print(exprs) => {
                let targets = exprs.iter().map(tell_expr).collect::<Vec<_>>().join(", ");
                table.add_row(vec![
                    Cell::new(format!("{}IO Side-Effect", prefix)).fg(Color::Blue),
                    Cell::new("stdout"),
                    Cell::new(format!("Print: {}", targets)),
                    Cell::new("Print").fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                table.add_row(vec![
                    Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                    Cell::new("if"),
                    Cell::new(format!("Condition: {}", tell_expr(condition))),
                    Cell::new("Branch").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, then_body, depth + 1);
                if let Some(else_branch) = else_body {
                    table.add_row(vec![
                        Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                        Cell::new("else"),
                        Cell::new("Fallback branch"),
                        Cell::new("Branch").fg(Color::DarkGrey),
                    ]);
                    traverse_statements(table, else_branch, depth + 1);
                }
            }
            AnalyzedStatement::While { condition, body } => {
                table.add_row(vec![
                    Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                    Cell::new("while"),
                    Cell::new(format!("Condition: {}", tell_expr(condition))),
                    Cell::new("Loop").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, body, depth + 1);
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                table.add_row(vec![
                    Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                    Cell::new("for"),
                    Cell::new(format!("Iterating over {}", tell_expr(iterator))),
                    Cell::new("Loop").fg(Color::DarkGrey),
                ]);
                table.add_row(vec![
                    Cell::new(format!("  {}Birth", prefix)).fg(Color::Green),
                    Cell::new(variable.as_str()),
                    Cell::new("Loop iteration variable"),
                    Cell::new("Inferred").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, body, depth + 1);
            }
            AnalyzedStatement::FunctionDef {
                name,
                params,
                return_type,
                ..
            } => {
                let params_str = params
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                table.add_row(vec![
                    Cell::new(format!("{}Birth", prefix)).fg(Color::Green),
                    Cell::new(name.as_str()),
                    Cell::new(format!("Function ({})", params_str)),
                    Cell::new(format!("{:?}", return_type)).fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::TypeDefinition { name, fields } => {
                let fields_str = fields
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                table.add_row(vec![
                    Cell::new(format!("{}Birth", prefix)).fg(Color::Green),
                    Cell::new(name.as_str()),
                    Cell::new(format!("Struct {{ {} }}", fields_str)),
                    Cell::new("Type").fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::TestDeclaration { name, body } => {
                table.add_row(vec![
                    Cell::new(format!("{}Test Definition", prefix)).fg(Color::Cyan),
                    Cell::new(name.as_str()),
                    Cell::new("Test declared"),
                    Cell::new("Verification").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, body, depth + 1);
            }
            _ => {
                // Ignore other statements for timeline
            }
        }
    }
}

// Simple formatter to avoid dragging in all of narrator
fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", tell_expr(owner), property)
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("{}({})", verb, args_str.join(", "))
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            format!("({} {:?} {})", tell_expr(left), op, tell_expr(right))
        }
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_variable_birth() {
        let source = "ξ 10 ἔστω.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let mut table = Table::new();
        traverse_statements(&mut table, &program.statements, 0);
        let output = table.to_string();

        assert!(output.contains("Birth"));
        assert!(output.contains('ξ'));
        assert!(output.contains("Initial value assigned"));
    }

    #[test]
    fn test_timeline_mutation() {
        let source = "ξ μετά 10 ἔστω. ξ 20 γίγνεται.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let mut table = Table::new();
        traverse_statements(&mut table, &program.statements, 0);
        let output = table.to_string();

        assert!(output.contains("Mutation"));
        assert!(output.contains("Value reassigned"));
        assert!(output.contains("Birth")); // Has birth too
    }

    #[test]
    fn test_timeline_full_coverage() {
        // Build AST manually to bypass parser/analyzer complexities for coverage of the formatter logic
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
        use smol_str::SmolStr;

        let stmts = vec![
            AnalyzedStatement::Print(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("hello".to_string()),
                glossa_type: GlossaType::String,
            }]),
            AnalyzedStatement::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![AnalyzedStatement::Break],
                else_body: Some(vec![AnalyzedStatement::Continue]),
            },
            AnalyzedStatement::While {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(false),
                    glossa_type: GlossaType::Boolean,
                }),
                body: vec![],
            },
            AnalyzedStatement::For {
                variable: SmolStr::new("x"),
                iterator: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(SmolStr::new("list")),
                    glossa_type: GlossaType::Unknown,
                }),
                body: vec![],
            },
            AnalyzedStatement::FunctionDef {
                name: SmolStr::new("my_func"),
                params: vec![(SmolStr::new("a"), Some(GlossaType::Number))],
                body: vec![],
                return_type: None,
            },
            AnalyzedStatement::TypeDefinition {
                name: SmolStr::new("Point"),
                fields: vec![(SmolStr::new("x"), GlossaType::Number)],
            },
            AnalyzedStatement::TestDeclaration {
                name: "my_test".to_string(),
                body: vec![],
            },
        ];

        let mut table = Table::new();
        traverse_statements(&mut table, &stmts, 0);
        let output = table.to_string();

        assert!(output.contains("IO Side-Effect")); // print
        assert!(output.contains("stdout"));
        assert!(output.contains("Control Flow Divergence"));
        assert!(output.contains("if"));
        assert!(output.contains("else"));
        assert!(output.contains("while"));
        assert!(output.contains("for"));
        assert!(output.contains("my_func")); // func def
        assert!(output.contains("Point")); // type def
        assert!(output.contains("my_test")); // test def
    }

    #[test]
    fn test_tell_expr_coverage() {
        use crate::morphology::lexicon::BinaryOp;
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
        use smol_str::SmolStr;

        // StringLiteral
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral("test".to_string()),
            glossa_type: GlossaType::String,
        };
        assert_eq!(tell_expr(&expr), "\"test\"");

        // NumberLiteral
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(42),
            glossa_type: GlossaType::Number,
        };
        assert_eq!(tell_expr(&expr), "42");

        // BooleanLiteral
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        };
        assert_eq!(tell_expr(&expr), "true");

        // Variable
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(SmolStr::new("x")),
            glossa_type: GlossaType::Number,
        };
        assert_eq!(tell_expr(&expr), "`x`");

        // PropertyAccess
        let owner = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(SmolStr::new("obj")),
            glossa_type: GlossaType::Unknown,
        });
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner,
                property: SmolStr::new("prop"),
            },
            glossa_type: GlossaType::Unknown,
        };
        assert_eq!(tell_expr(&expr), "`obj`.prop");

        // VerbCall
        let arg = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::VerbCall {
                verb: SmolStr::new("func"),
                args: vec![arg],
            },
            glossa_type: GlossaType::Unknown,
        };
        assert_eq!(tell_expr(&expr), "func(1)");

        // BinOp
        let left = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        });
        let right = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(2),
            glossa_type: GlossaType::Number,
        });
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left,
                op: BinaryOp::Add,
                right,
            },
            glossa_type: GlossaType::Number,
        };
        assert_eq!(tell_expr(&expr), "(1 Add 2)");

        // Unhandled variant
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: GlossaType::Unknown,
        };
        assert_eq!(tell_expr(&expr), "<expr>");
    }

    #[test]
    fn test_run_timeline_integration() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_timeline.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all("«χαῖρε» λέγε.".as_bytes()).unwrap();
        }

        let result = run_timeline(&file_path);
        assert!(result.is_ok());
    }
}
