//! The Philosopher Tool (ὁ Φιλόσοφος)
//!
//! This module implements a thematic static analyzer that acts as a philosophical mentor.
//! It traverses the `AnalyzedProgram` AST to identify "code smells" such as deep nesting,
//! overly long functions, or unused bindings, and presents these observations as Ancient
//! Greek philosophical maxims.
//!
//! # The Philosophy
//!
//! "Socrates said 'Wisdom is knowing what you don't know', but this function tries to know too much."
//!
//! # How it works
//!
//! The `run_philosopher` function parses and analyzes a Glossa file, producing an `AnalyzedProgram`.
//! Then, the `Philosopher` struct recursively walks the semantic tree to collect `Maxim`s (warnings).
//! Finally, these maxims are presented to the user.

use crate::parser::parse;
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, analyze_program,
};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Represents a philosophical warning or observation.
#[derive(Debug, Clone)]
pub struct Maxim {
    pub location: String,
    pub observation: String,
    pub quote: String,
}

/// The main entry point for the Philosopher tool.
pub fn run_philosopher(input: &Path) -> Result<()> {
    let status = crate::tools::ui::Status::start_with_symbol("Στοχασμός (Contemplating)", "🦉");

    let source = crate::tools::runner::load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let mut philosopher = Philosopher::new();
    philosopher.contemplate(&analyzed);

    status.success();

    let maxims = philosopher.get_maxims();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   P H I L O S O P H E R".bold().cyan());
    println!("   {}", "The Oracle of Code Quality".italic().dim());
    println!();

    if maxims.is_empty() {
        println!(
            "   {}",
            "✨ Your logic is as pure as Platonic forms. No flaws found.".green()
        );
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Τόπος (Location)")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Yellow),
                Cell::new("Παρατήρησις (Observation)")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Cyan),
                Cell::new("Γνωμικόν (Maxim)")
                    .add_attribute(Attribute::Bold)
                    .fg(Color::Magenta),
            ]);

        for maxim in maxims {
            table.add_row(vec![
                Cell::new(&maxim.location),
                Cell::new(&maxim.observation),
                Cell::new(&maxim.quote).add_attribute(Attribute::Italic),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}

/// The stateful analyzer that traverses the AST to collect maxims.
pub struct Philosopher {
    maxims: Vec<Maxim>,
}

impl Default for Philosopher {
    fn default() -> Self {
        Self::new()
    }
}

impl Philosopher {
    pub fn new() -> Self {
        Self { maxims: Vec::new() }
    }

    pub fn get_maxims(&self) -> &[Maxim] {
        &self.maxims
    }

    pub fn contemplate(&mut self, program: &AnalyzedProgram) {
        for stmt in &program.statements {
            self.visit_statement(stmt, 0, "Global");
        }

        // Also check functions in scope
        for func in program.scope.functions() {
            // we don't have body in FunctionSignature, it's defined in AnalyzedStatement::FunctionDef
            // so we skip body check here and only check param_types
            let context = format!("Function `{}`", func.name);
            if func.param_types.len() > 4 {
                self.maxims.push(Maxim {
                    location: context.clone(),
                    observation: format!("Function has {} parameters.", func.param_types.len()),
                    quote: "Epictetus reminds us 'Wealth consists not in having great possessions, but in having few wants.' Does this function truly need so many inputs?".to_string(),
                });
            }
        }
    }

    fn visit_statement(&mut self, stmt: &AnalyzedStatement, depth: usize, context: &str) {
        if depth > 3 {
            self.maxims.push(Maxim {
                location: context.to_string(),
                observation: format!("Deeply nested logic detected (depth {}).", depth),
                quote: "Like Daedalus' labyrinth, this logic twists too deeply. Break it apart lest the Minotaur of bugs devours you.".to_string(),
            });
        }

        match stmt {
            AnalyzedStatement::Binding { value, .. } => {
                self.visit_expr(value, context);
            }
            AnalyzedStatement::Assignment { value, .. } => {
                self.visit_expr(value, context);
            }
            AnalyzedStatement::Print(exprs)
            | AnalyzedStatement::Expression(exprs)
            | AnalyzedStatement::Query(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr, context);
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.visit_expr(condition, context);
                for s in then_body {
                    self.visit_statement(s, depth + 1, context);
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.visit_statement(s, depth + 1, context);
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                self.visit_expr(condition, context);
                for s in body {
                    self.visit_statement(s, depth + 1, context);
                }
            }
            AnalyzedStatement::For { iterator, body, .. } => {
                self.visit_expr(iterator, context);
                for s in body {
                    self.visit_statement(s, depth + 1, context);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee, context);
                for (pat, body) in arms {
                    self.visit_expr(pat, context);
                    for s in body {
                        self.visit_statement(s, depth + 1, context);
                    }
                }
            }
            AnalyzedStatement::FunctionDef {
                name, params, body, ..
            } => {
                let func_context = format!("Function `{}`", name);
                if params.len() > 4 {
                    self.maxims.push(Maxim {
                        location: func_context.clone(),
                        observation: format!("Function has {} parameters.", params.len()),
                        quote: "Epictetus reminds us 'Wealth consists not in having great possessions, but in having few wants.' Does this function truly need so many inputs?".to_string(),
                    });
                }
                if body.len() > 10 {
                    self.maxims.push(Maxim {
                        location: func_context.clone(),
                        observation: format!("Function contains {} statements.", body.len()),
                        quote: "Aristotle taught that moderation is key. A function, like a good argument, should not be overly long.".to_string(),
                    });
                }
                for s in body {
                    self.visit_statement(s, depth + 1, &func_context);
                }
            }
            AnalyzedStatement::TestDeclaration { body, name } => {
                let test_context = format!("Test `{}`", name);
                for s in body {
                    self.visit_statement(s, depth + 1, &test_context);
                }
            }
            AnalyzedStatement::Return { value: Some(v) } => {
                self.visit_expr(v, context);
            }
            AnalyzedStatement::Return { value: None } => {}
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr, _context: &str) {
        match &expr.expr {
            AnalyzedExprKind::PropertyAccess { owner, .. } => self.visit_expr(owner, _context),
            AnalyzedExprKind::VerbCall { args, .. } => {
                for arg in args {
                    self.visit_expr(arg, _context);
                }
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.visit_expr(left, _context);
                self.visit_expr(right, _context);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => self.visit_expr(operand, _context),
            AnalyzedExprKind::Range { start, end, .. } => {
                self.visit_expr(start, _context);
                self.visit_expr(end, _context);
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.visit_expr(e, _context);
                }
            }
            AnalyzedExprKind::Some(e)
            | AnalyzedExprKind::Ok(e)
            | AnalyzedExprKind::Err(e)
            | AnalyzedExprKind::Unwrap(e)
            | AnalyzedExprKind::Try(e) => {
                self.visit_expr(e, _context);
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.visit_expr(array, _context);
                self.visit_expr(index, _context);
            }
            AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.visit_expr(arg, _context);
                }
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. }
            | AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
                self.visit_expr(receiver, _context);
                for arg in args {
                    self.visit_expr(arg, _context);
                }
            }
            AnalyzedExprKind::Lambda { body, .. } => {
                self.visit_expr(body, _context);
            }
            AnalyzedExprKind::Assert { condition } => self.visit_expr(condition, _context),
            AnalyzedExprKind::AssertEq { left, right } => {
                self.visit_expr(left, _context);
                self.visit_expr(right, _context);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedProgram, GlossaType, Scope};

    fn dummy_expr() -> Box<AnalyzedExpr> {
        Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        })
    }

    #[test]
    fn test_philosopher_deep_nesting() {
        // Build an artificially deeply nested AST
        let mut inner_stmts = vec![AnalyzedStatement::Break];

        for _ in 0..5 {
            inner_stmts = vec![AnalyzedStatement::If {
                condition: dummy_expr(),
                then_body: inner_stmts,
                else_body: None,
            }];
        }

        let program = AnalyzedProgram {
            statements: inner_stmts,
            scope: Scope::new(),
        };

        let mut philosopher = Philosopher::new();
        philosopher.contemplate(&program);

        let maxims = philosopher.get_maxims();
        assert!(
            !maxims.is_empty(),
            "Philosopher should have found a deep nesting maxim"
        );
        assert!(
            maxims
                .iter()
                .any(|m| m.observation.contains("Deeply nested logic detected")),
            "Did not find the expected deep nesting observation."
        );
    }

    #[test]
    fn test_philosopher_visit_statement_coverage() {
        let stmts = vec![
            AnalyzedStatement::Binding {
                name: "x".into(),
                value: *dummy_expr(),
                mutable: false,
            },
            AnalyzedStatement::Assignment {
                name: "x".into(),
                value: *dummy_expr(),
            },
            AnalyzedStatement::Print(vec![*dummy_expr()]),
            AnalyzedStatement::Expression(vec![*dummy_expr()]),
            AnalyzedStatement::Query(vec![*dummy_expr()]),
            AnalyzedStatement::If {
                condition: dummy_expr(),
                then_body: vec![AnalyzedStatement::Break],
                else_body: Some(vec![AnalyzedStatement::Continue]),
            },
            AnalyzedStatement::While {
                condition: dummy_expr(),
                body: vec![AnalyzedStatement::Break],
            },
            AnalyzedStatement::For {
                variable: "i".into(),
                iterator: dummy_expr(),
                body: vec![AnalyzedStatement::Break],
            },
            AnalyzedStatement::Match {
                scrutinee: dummy_expr(),
                arms: vec![(*dummy_expr(), vec![AnalyzedStatement::Break])],
            },
            AnalyzedStatement::FunctionDef {
                name: "test".into(),
                params: vec![
                    ("a".into(), None),
                    ("b".into(), None),
                    ("c".into(), None),
                    ("d".into(), None),
                    ("e".into(), None),
                ],
                body: vec![
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                    AnalyzedStatement::Break,
                ],
                return_type: None,
            },
            AnalyzedStatement::TestDeclaration {
                name: "test".into(),
                body: vec![AnalyzedStatement::Break],
            },
            AnalyzedStatement::Return {
                value: Some(dummy_expr()),
            },
            AnalyzedStatement::Return { value: None },
            AnalyzedStatement::Break,
            AnalyzedStatement::Continue,
            AnalyzedStatement::TypeDefinition {
                name: "User".into(),
                fields: vec![],
            },
            AnalyzedStatement::TraitDefinition {
                name: "Show".into(),
                methods: vec![],
            },
            AnalyzedStatement::TraitImplementation {
                trait_name: "Show".into(),
                type_name: "User".into(),
                methods: vec![],
            },
        ];

        let program = AnalyzedProgram {
            statements: stmts,
            scope: Scope::new(),
        };

        let mut philosopher = Philosopher::new();
        philosopher.contemplate(&program);

        let maxims = philosopher.get_maxims();
        assert!(
            maxims
                .iter()
                .any(|m| m.observation.contains("Function has 5 parameters")),
            "Did not find the expected many parameters observation."
        );
        assert!(
            maxims
                .iter()
                .any(|m| m.observation.contains("Function contains 11 statements")),
            "Did not find the expected many statements observation."
        );
    }

    #[test]
    fn test_philosopher_visit_expr_coverage() {
        let exprs = vec![
            AnalyzedExprKind::PropertyAccess {
                owner: dummy_expr(),
                property: "name".into(),
            },
            AnalyzedExprKind::VerbCall {
                verb: "say".into(),
                args: vec![*dummy_expr()],
            },
            AnalyzedExprKind::BinOp {
                left: dummy_expr(),
                op: crate::morphology::lexicon::BinaryOp::Add,
                right: dummy_expr(),
            },
            AnalyzedExprKind::UnaryOp {
                op: crate::morphology::lexicon::UnaryOp::Not,
                operand: dummy_expr(),
            },
            AnalyzedExprKind::Range {
                start: dummy_expr(),
                end: dummy_expr(),
                inclusive: false,
            },
            AnalyzedExprKind::ArrayLiteral(vec![*dummy_expr()]),
            AnalyzedExprKind::Some(dummy_expr()),
            AnalyzedExprKind::Ok(dummy_expr()),
            AnalyzedExprKind::Err(dummy_expr()),
            AnalyzedExprKind::Unwrap(dummy_expr()),
            AnalyzedExprKind::Try(dummy_expr()),
            AnalyzedExprKind::IndexAccess {
                array: dummy_expr(),
                index: dummy_expr(),
            },
            AnalyzedExprKind::FunctionCall {
                func: "func".into(),
                args: vec![*dummy_expr()],
            },
            AnalyzedExprKind::StructInstantiation {
                type_name: "User".into(),
                fields: vec![],
                args: vec![*dummy_expr()],
            },
            AnalyzedExprKind::MethodCall {
                receiver: dummy_expr(),
                method: "push".into(),
                args: vec![*dummy_expr()],
            },
            AnalyzedExprKind::TraitMethodCall {
                receiver: dummy_expr(),
                trait_name: "Show".into(),
                method_name: "print".into(),
                args: vec![*dummy_expr()],
            },
            AnalyzedExprKind::Lambda {
                params: vec!["x".into()],
                body: dummy_expr(),
                capture_mode: crate::semantic::CaptureMode::Borrow,
            },
            AnalyzedExprKind::Assert {
                condition: dummy_expr(),
            },
            AnalyzedExprKind::AssertEq {
                left: dummy_expr(),
                right: dummy_expr(),
            },
            AnalyzedExprKind::StringLiteral("hello".into()),
            AnalyzedExprKind::NumberLiteral(42),
            AnalyzedExprKind::Variable("x".into()),
            AnalyzedExprKind::None,
            AnalyzedExprKind::CollectionNew {
                collection_type: "HashSet".into(),
            },
        ];

        let mut philosopher = Philosopher::new();
        for expr in exprs {
            let analyzed_expr = AnalyzedExpr {
                expr,
                glossa_type: GlossaType::Unknown,
            };
            philosopher.visit_expr(&analyzed_expr, "Test");
        }
    }

    #[test]
    fn test_philosopher_function_params() {
        let mut scope = Scope::new();
        // Add a function with 5 parameters to scope to trigger the warning
        scope.define_function(
            "complex_function",
            vec![
                GlossaType::Unknown,
                GlossaType::Unknown,
                GlossaType::Unknown,
                GlossaType::Unknown,
                GlossaType::Unknown,
            ],
            None,
        );

        let program = AnalyzedProgram {
            statements: vec![],
            scope,
        };

        let mut philosopher = Philosopher::new();
        philosopher.contemplate(&program);

        let maxims = philosopher.get_maxims();
        assert!(
            maxims
                .iter()
                .any(|m| m.observation.contains("Function has 5 parameters")),
            "Did not find the expected many parameters observation."
        );
    }
}
