//! The Critic Tool ("Critic")
//!
//! This module implements the "Critic" functionality, which acts as a static linter
//! for ΓΛΩΣΣΑ programs. It analyzes the `AnalyzedProgram` to detect "Code Smells"
//! (Οσμαὶ Κώδικος) such as deep nesting, god functions, complex conditions, and empty blocks.

use crate::parser::parse;
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, analyze_program,
};
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// A detected code smell
pub struct CodeSmell {
    pub kind: String,
    pub description: String,
    pub context: String,
}

/// Run the Critic tool on a file
pub fn run_critic(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Κριτική (Linting)", "🧐");

    let source = crate::tools::runner::load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let mut critic = Critic::new();
    critic.analyze(&program);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   C R I T I C".bold().cyan());
    println!("   {}", "Static Code Analysis".italic().dim());
    println!();

    if critic.smells.is_empty() {
        println!(
            "   {}",
            "✨ No code smells detected. The code is pure."
                .green()
                .bold()
        );
    } else {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);
        table.set_header(vec![
            Cell::new("Type")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new("Description").add_attribute(Attribute::Bold),
            Cell::new("Location/Context")
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkGrey),
        ]);

        for smell in &critic.smells {
            table.add_row(vec![
                Cell::new(&smell.kind).fg(Color::Yellow),
                Cell::new(&smell.description),
                Cell::new(&smell.context).fg(Color::DarkGrey),
            ]);
        }
        println!("{table}");
    }
    println!();

    Ok(())
}

/// The Critic linter engine
pub struct Critic {
    pub smells: Vec<CodeSmell>,
}

impl Default for Critic {
    fn default() -> Self {
        Self::new()
    }
}

impl Critic {
    pub fn new() -> Self {
        Self { smells: Vec::new() }
    }

    pub fn analyze(&mut self, program: &AnalyzedProgram) {
        for stmt in &program.statements {
            self.visit_statement(stmt, 0);
        }
    }

    fn add_smell(&mut self, kind: &str, description: &str, context: &str) {
        self.smells.push(CodeSmell {
            kind: kind.to_string(),
            description: description.to_string(),
            context: context.to_string(),
        });
    }

    fn visit_statement(&mut self, stmt: &AnalyzedStatement, depth: usize) {
        // Rule: Deep Nesting (> 3)
        if depth > 3 {
            self.add_smell(
                "Deep Nesting",
                "Nesting depth exceeds 3. Consider refactoring into functions.",
                &format!("Depth: {}", depth),
            );
        }

        match stmt {
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                // Rule: Empty Blocks
                if then_body.is_empty() {
                    self.add_smell(
                        "Empty Block",
                        "The 'if' condition has an empty body.",
                        "If statement",
                    );
                }

                // Rule: Complex Conditions
                let op_count = self.count_binary_ops(condition);
                if op_count > 3 {
                    self.add_smell(
                        "Complex Condition",
                        &format!(
                            "Condition has {} binary operators. Consider extracting to a variable.",
                            op_count
                        ),
                        "If statement",
                    );
                }

                self.visit_expr(condition);
                for s in then_body {
                    self.visit_statement(s, depth + 1);
                }
                if let Some(eb) = else_body {
                    if eb.is_empty() {
                        self.add_smell(
                            "Empty Block",
                            "The 'else' condition has an empty body.",
                            "Else statement",
                        );
                    }
                    for s in eb {
                        self.visit_statement(s, depth + 1);
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                if body.is_empty() {
                    self.add_smell(
                        "Empty Block",
                        "The 'while' loop has an empty body.",
                        "While loop",
                    );
                }

                let op_count = self.count_binary_ops(condition);
                if op_count > 3 {
                    self.add_smell(
                        "Complex Condition",
                        &format!(
                            "Condition has {} binary operators. Consider extracting to a variable.",
                            op_count
                        ),
                        "While statement",
                    );
                }

                self.visit_expr(condition);
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::For { iterator, body, .. } => {
                if body.is_empty() {
                    self.add_smell(
                        "Empty Block",
                        "The 'for' loop has an empty body.",
                        "For loop",
                    );
                }
                self.visit_expr(iterator);
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee);
                for (pat, body) in arms {
                    if body.is_empty() {
                        self.add_smell(
                            "Empty Block",
                            "A 'match' arm has an empty body.",
                            "Match statement",
                        );
                    }
                    self.visit_expr(pat);
                    for s in body {
                        self.visit_statement(s, depth + 1);
                    }
                }
            }
            AnalyzedStatement::FunctionDef {
                name, params, body, ..
            } => {
                // Rule: God Functions
                if params.len() > 5 {
                    self.add_smell(
                        "God Function",
                        &format!(
                            "Function has {} parameters. Consider grouping into a struct.",
                            params.len()
                        ),
                        &format!("Function '{}'", name),
                    );
                }
                if body.len() > 20 {
                    self.add_smell(
                        "God Function",
                        &format!(
                            "Function has {} statements. Consider breaking it down.",
                            body.len()
                        ),
                        &format!("Function '{}'", name),
                    );
                }

                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::Binding { value, .. }
            | AnalyzedStatement::Assignment { value, .. } => {
                self.visit_expr(value);
            }
            AnalyzedStatement::Print(exprs)
            | AnalyzedStatement::Query(exprs)
            | AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::Return { value: Some(v) } => {
                self.visit_expr(v);
            }
            AnalyzedStatement::TraitImplementation { methods, .. } => {
                for method in methods {
                    if method.params.len() > 5 {
                        self.add_smell(
                            "God Function",
                            &format!(
                                "Trait method has {} parameters. Consider grouping into a struct.",
                                method.params.len()
                            ),
                            &format!("Method '{}'", method.name),
                        );
                    }
                    if let Some(body) = &method.body {
                        if body.len() > 20 {
                            self.add_smell(
                                "God Function",
                                &format!(
                                    "Trait method has {} statements. Consider breaking it down.",
                                    body.len()
                                ),
                                &format!("Method '{}'", method.name),
                            );
                        }
                        for s in body {
                            self.visit_statement(s, depth + 1);
                        }
                    }
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                if body.is_empty() {
                    self.add_smell(
                        "Empty Block",
                        "The test declaration has an empty body.",
                        "Test declaration",
                    );
                }
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            // Ignore other definitions
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr) {
        match &expr.expr {
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => self.visit_expr(operand),
            AnalyzedExprKind::PropertyAccess { owner, .. } => self.visit_expr(owner),
            AnalyzedExprKind::MethodCall { receiver, args, .. }
            | AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
                self.visit_expr(receiver);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::VerbCall { args, .. }
            | AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.visit_expr(start);
                self.visit_expr(end);
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            }
            AnalyzedExprKind::Some(e)
            | AnalyzedExprKind::Ok(e)
            | AnalyzedExprKind::Err(e)
            | AnalyzedExprKind::Try(e)
            | AnalyzedExprKind::Unwrap(e) => {
                self.visit_expr(e);
            }
            AnalyzedExprKind::Lambda { body, .. } => self.visit_expr(body),
            AnalyzedExprKind::Assert { condition } => self.visit_expr(condition),
            AnalyzedExprKind::AssertEq { left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            _ => {}
        }
    }

    fn count_binary_ops(&self, expr: &AnalyzedExpr) -> usize {
        let mut count = 0;
        self.count_binary_ops_recursive(expr, &mut count);
        count
    }

    fn count_binary_ops_recursive(&self, expr: &AnalyzedExpr, count: &mut usize) {
        match &expr.expr {
            AnalyzedExprKind::BinOp { left, right, .. } => {
                *count += 1;
                self.count_binary_ops_recursive(left, count);
                self.count_binary_ops_recursive(right, count);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.count_binary_ops_recursive(operand, count)
            }
            AnalyzedExprKind::PropertyAccess { owner, .. } => {
                self.count_binary_ops_recursive(owner, count)
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. }
            | AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
                self.count_binary_ops_recursive(receiver, count);
                for arg in args {
                    self.count_binary_ops_recursive(arg, count);
                }
            }
            AnalyzedExprKind::VerbCall { args, .. }
            | AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.count_binary_ops_recursive(arg, count);
                }
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.count_binary_ops_recursive(e, count);
                }
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.count_binary_ops_recursive(start, count);
                self.count_binary_ops_recursive(end, count);
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.count_binary_ops_recursive(array, count);
                self.count_binary_ops_recursive(index, count);
            }
            AnalyzedExprKind::Some(e)
            | AnalyzedExprKind::Ok(e)
            | AnalyzedExprKind::Err(e)
            | AnalyzedExprKind::Try(e)
            | AnalyzedExprKind::Unwrap(e) => {
                self.count_binary_ops_recursive(e, count);
            }
            AnalyzedExprKind::Lambda { body, .. } => self.count_binary_ops_recursive(body, count),
            AnalyzedExprKind::Assert { condition } => {
                self.count_binary_ops_recursive(condition, count)
            }
            AnalyzedExprKind::AssertEq { left, right } => {
                self.count_binary_ops_recursive(left, count);
                self.count_binary_ops_recursive(right, count);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedMethod, GlossaType, Scope};

    #[test]
    fn test_deep_nesting() {
        // Construct a deep nesting scenario
        let stmt = AnalyzedStatement::If {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
            then_body: vec![AnalyzedStatement::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![AnalyzedStatement::If {
                    condition: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::BooleanLiteral(true),
                        glossa_type: GlossaType::Boolean,
                    }),
                    then_body: vec![AnalyzedStatement::If {
                        condition: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(true),
                            glossa_type: GlossaType::Boolean,
                        }),
                        then_body: vec![AnalyzedStatement::Break],
                        else_body: None,
                    }],
                    else_body: None,
                }],
                else_body: None,
            }],
            else_body: None,
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let mut critic = Critic::new();
        critic.analyze(&program);

        assert!(critic.smells.iter().any(|s| s.kind == "Deep Nesting"));
    }

    #[test]
    fn test_god_function() {
        let stmt = AnalyzedStatement::FunctionDef {
            name: "big_func".into(),
            params: vec![
                ("a".into(), None),
                ("b".into(), None),
                ("c".into(), None),
                ("d".into(), None),
                ("e".into(), None),
                ("f".into(), None),
            ],
            return_type: None,
            body: vec![AnalyzedStatement::Break],
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let mut critic = Critic::new();
        critic.analyze(&program);

        assert!(
            critic
                .smells
                .iter()
                .any(|s| s.kind == "God Function" && s.description.contains("parameters"))
        );
    }

    #[test]
    fn test_empty_blocks() {
        let stmt = AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
            body: vec![],
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let mut critic = Critic::new();
        critic.analyze(&program);

        assert!(critic.smells.iter().any(|s| s.kind == "Empty Block"));
    }

    #[test]
    fn test_complex_condition() {
        let condition = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BinOp {
                        left: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::BinOp {
                                left: Box::new(AnalyzedExpr {
                                    expr: AnalyzedExprKind::BinOp {
                                        left: Box::new(AnalyzedExpr {
                                            expr: AnalyzedExprKind::BooleanLiteral(true),
                                            glossa_type: GlossaType::Boolean,
                                        }),
                                        op: crate::morphology::lexicon::BinaryOp::And,
                                        right: Box::new(AnalyzedExpr {
                                            expr: AnalyzedExprKind::BooleanLiteral(true),
                                            glossa_type: GlossaType::Boolean,
                                        }),
                                    },
                                    glossa_type: GlossaType::Boolean,
                                }),
                                op: crate::morphology::lexicon::BinaryOp::And,
                                right: Box::new(AnalyzedExpr {
                                    expr: AnalyzedExprKind::BooleanLiteral(true),
                                    glossa_type: GlossaType::Boolean,
                                }),
                            },
                            glossa_type: GlossaType::Boolean,
                        }),
                        op: crate::morphology::lexicon::BinaryOp::And,
                        right: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(true),
                            glossa_type: GlossaType::Boolean,
                        }),
                    },
                    glossa_type: GlossaType::Boolean,
                }),
                op: crate::morphology::lexicon::BinaryOp::And,
                right: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
            },
            glossa_type: GlossaType::Boolean,
        };

        let stmt = AnalyzedStatement::If {
            condition: Box::new(condition),
            then_body: vec![AnalyzedStatement::Break],
            else_body: None,
        };

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let mut critic = Critic::new();
        critic.analyze(&program);

        assert!(critic.smells.iter().any(|s| s.kind == "Complex Condition"));
    }

    #[test]
    fn test_run_critic() {
        // Create a temporary file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_critic.gl");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all("«χαῖρε» λέγε.".as_bytes()).unwrap();
        }

        let result = run_critic(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_critic_with_smells() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_critic_smells.gl");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all("εἰ ἀληθές ἐστι, { }.".as_bytes()).unwrap(); // empty block smell, syntactically valid
        }

        let result = run_critic(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_critic_coverage() {
        let stmt1 = AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Range {
                    start: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    end: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(10),
                        glossa_type: GlossaType::Number,
                    }),
                    inclusive: false,
                },
                glossa_type: GlossaType::Unknown,
            }),
            body: vec![],
        };

        let stmt2 = AnalyzedStatement::Match {
            scrutinee: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            arms: vec![(
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                },
                vec![],
            )],
        };

        let stmt3 = AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![],
        };

        let stmt4 = AnalyzedStatement::TraitImplementation {
            trait_name: "Trait".into(),
            type_name: "Type".into(),
            methods: vec![AnalyzedMethod {
                name: "method".into(),
                params: vec![
                    ("a".into(), GlossaType::Unknown),
                    ("b".into(), GlossaType::Unknown),
                    ("c".into(), GlossaType::Unknown),
                    ("d".into(), GlossaType::Unknown),
                    ("e".into(), GlossaType::Unknown),
                    ("f".into(), GlossaType::Unknown),
                ],
                body: Some(vec![AnalyzedStatement::Break]),
                return_type: None,
            }],
        };

        let exprs = AnalyzedStatement::Expression(vec![
            AnalyzedExpr {
                expr: AnalyzedExprKind::UnaryOp {
                    op: crate::morphology::lexicon::UnaryOp::Not,
                    operand: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::BooleanLiteral(true),
                        glossa_type: GlossaType::Boolean,
                    }),
                },
                glossa_type: GlossaType::Boolean,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::StringLiteral("test".into()),
                        glossa_type: GlossaType::String,
                    }),
                    property: "len".into(),
                },
                glossa_type: GlossaType::Number,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    method: "method".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::FunctionCall {
                    func: "func".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                        glossa_type: GlossaType::Unknown,
                    }),
                    index: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(0),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Lambda {
                    params: vec![],
                    body: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    capture_mode: crate::semantic::CaptureMode::Borrow,
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Assert {
                    condition: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::BooleanLiteral(true),
                        glossa_type: GlossaType::Boolean,
                    }),
                },
                glossa_type: GlossaType::Unit,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::AssertEq {
                    left: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Unit,
            },
        ]);

        let exprs2 = AnalyzedStatement::Expression(vec![
            AnalyzedExpr {
                expr: AnalyzedExprKind::TraitMethodCall {
                    receiver: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    trait_name: "Trait".into(),
                    method_name: "method".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::VerbCall {
                    verb: "verb".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::StructInstantiation {
                    type_name: "Struct".into(),
                    fields: vec![],
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
        ]);

        let stmt_if = AnalyzedStatement::If {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(false),
                glossa_type: GlossaType::Boolean,
            }),
            then_body: vec![AnalyzedStatement::Break],
            else_body: Some(vec![AnalyzedStatement::Break]),
        };

        let stmt_while = AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(false),
                glossa_type: GlossaType::Boolean,
            }),
            body: vec![AnalyzedStatement::Break],
        };

        let stmt_for = AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            body: vec![AnalyzedStatement::Break],
        };

        let stmt_match = AnalyzedStatement::Match {
            scrutinee: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            arms: vec![(
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                },
                vec![AnalyzedStatement::Break],
            )],
        };

        let stmt_test = AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![AnalyzedStatement::Break],
        };

        let stmt_binding = AnalyzedStatement::Binding {
            name: "x".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            },
            mutable: false,
        };

        let stmt_print = AnalyzedStatement::Print(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        }]);

        let stmt_return = AnalyzedStatement::Return {
            value: Some(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            })),
        };

        let program = AnalyzedProgram {
            statements: vec![
                stmt1,
                stmt2,
                stmt3,
                stmt4,
                exprs,
                exprs2,
                stmt_if,
                stmt_while,
                stmt_for,
                stmt_match,
                stmt_test,
                stmt_binding,
                stmt_print,
                stmt_return,
            ],
            scope: Scope::new(),
        };

        let mut critic = Critic::new();
        critic.analyze(&program);

        assert!(
            critic
                .smells
                .iter()
                .any(|s| s.kind == "Empty Block" && s.context == "For loop")
        );
        assert!(
            critic
                .smells
                .iter()
                .any(|s| s.kind == "Empty Block" && s.context == "Match statement")
        );
        assert!(
            critic
                .smells
                .iter()
                .any(|s| s.kind == "Empty Block" && s.context == "Test declaration")
        );
        assert!(
            critic
                .smells
                .iter()
                .any(|s| s.kind == "God Function" && s.context == "Method 'method'")
        );

        // Trigger count_binary_ops explicitly on nodes to get coverage there as well
        let mut count = 0;
        for stmt in &program.statements {
            if let AnalyzedStatement::Expression(exprs) = stmt {
                for expr in exprs {
                    critic.count_binary_ops_recursive(expr, &mut count);
                }
            }
        }
    }
}
