//! The Chronicle (ὁ Χρονογράφος) - Variable Lifecycle Tracker
//!
//! Tracks variables state changes (mutations) throughout the program.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::path::Path;

pub fn run_chronicle(input: &Path) -> Result<()> {
    let source = crate::tools::runner::load_source(input)?;
    let status =
        crate::tools::ui::Status::start_with_symbol("Χρονογράφος (Tracking History)", "📜");
    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };
    status.success();

    let mut visitor = ChronicleVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }

    use std::io::IsTerminal;
    let is_tty = std::io::stdout().is_terminal();

    if is_tty {
        println!();
        println!("   {}", "Γ Λ Ω Σ Σ Α   C H R O N I C L E".cyan().bold());
        println!("   {}", "Variable Lifecycle History".italic().dim());
        println!();

        let mut table = Table::new();
        table.load_preset(UTF8_FULL).set_header(vec![
            Cell::new("Variable")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new("Lifecycle").add_attribute(Attribute::Bold),
        ]);

        let mut keys: Vec<_> = visitor.history.keys().collect();
        keys.sort();
        for k in keys {
            let events = visitor.history.get(k).unwrap().join(" ➔ ");
            table.add_row(vec![Cell::new(k.as_str()), Cell::new(events)]);
        }
        println!("{table}");
    } else {
        let mut keys: Vec<_> = visitor.history.keys().collect();
        keys.sort();
        for k in keys {
            let events = visitor.history.get(k).unwrap().join(" -> ");
            println!("{}: {}", k, events);
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct ChronicleVisitor {
    pub history: FxHashMap<SmolStr, Vec<String>>,
}

impl ChronicleVisitor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                self.history
                    .entry(name.clone())
                    .or_default()
                    .push("Born".to_string());
                self.visit_expr(value);
            }
            AnalyzedStatement::Assignment { name, value } => {
                self.history
                    .entry(name.clone())
                    .or_default()
                    .push("Mutated".to_string());
                self.visit_expr(value);
            }
            AnalyzedStatement::Print(exprs)
            | AnalyzedStatement::Query(exprs)
            | AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.visit_expr(condition);
                for s in then_body {
                    self.visit_statement(s);
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                self.visit_expr(condition);
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::For { iterator, body, .. } => {
                self.visit_expr(iterator);
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee);
                for (pat, body) in arms {
                    self.visit_expr(pat);
                    for s in body {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::Return { value: Some(val) } => {
                self.visit_expr(val);
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::TraitImplementation { methods, .. } => {
                for m in methods {
                    if let Some(b) = &m.body {
                        for s in b {
                            self.visit_statement(s);
                        }
                    }
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            _ => {}
        }
    }
    pub fn visit_expr(&mut self, expr: &AnalyzedExpr) {
        match &expr.expr {
            AnalyzedExprKind::Variable(name) => {
                self.history
                    .entry(name.clone())
                    .or_default()
                    .push("Read".to_string());
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            AnalyzedExprKind::PropertyAccess { owner, .. } => {
                self.visit_expr(owner);
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. } => {
                self.visit_expr(receiver);
                for a in args {
                    self.visit_expr(a);
                }
            }
            AnalyzedExprKind::FunctionCall { args, .. } => {
                for a in args {
                    self.visit_expr(a);
                }
            }
            AnalyzedExprKind::StructInstantiation { args, .. } => {
                for a in args {
                    self.visit_expr(a);
                }
            }
            AnalyzedExprKind::ArrayLiteral(args) => {
                for a in args {
                    self.visit_expr(a);
                }
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            }
            AnalyzedExprKind::Lambda { body, .. } => {
                self.visit_expr(body);
            }
            AnalyzedExprKind::VerbCall { args, .. } => {
                for a in args {
                    self.visit_expr(a);
                }
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.visit_expr(start);
                self.visit_expr(end);
            }
            AnalyzedExprKind::Some(v)
            | AnalyzedExprKind::Ok(v)
            | AnalyzedExprKind::Err(v)
            | AnalyzedExprKind::Unwrap(v)
            | AnalyzedExprKind::Try(v) => {
                self.visit_expr(v);
            }
            AnalyzedExprKind::Assert { condition } => {
                self.visit_expr(condition);
            }
            AnalyzedExprKind::AssertEq { left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::GlossaType;

    #[test]
    fn test_chronicle_lifecycle() {
        let mut visitor = ChronicleVisitor::new();
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };
        let stmt1 = AnalyzedStatement::Binding {
            name: "x".into(),
            value: expr.clone(),
            mutable: true,
        };
        let stmt2 = AnalyzedStatement::Assignment {
            name: "x".into(),
            value: expr.clone(),
        };
        let stmt3 = AnalyzedStatement::Print(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Number,
        }]);

        visitor.visit_statement(&stmt1);
        visitor.visit_statement(&stmt2);
        visitor.visit_statement(&stmt3);

        let hist = visitor.history.get("x").unwrap();
        assert_eq!(hist.len(), 3);
        assert_eq!(hist[0], "Born");
        assert_eq!(hist[1], "Mutated");
        assert_eq!(hist[2], "Read");
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::semantic::GlossaType;

    #[test]
    fn test_chronicle_coverage() {
        let mut visitor = ChronicleVisitor::new();

        let dummy_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        // Expression
        visitor.visit_statement(&AnalyzedStatement::Expression(vec![dummy_expr.clone()]));

        // If
        visitor.visit_statement(&AnalyzedStatement::If {
            condition: Box::new(dummy_expr.clone()),
            then_body: vec![AnalyzedStatement::Break],
            else_body: Some(vec![AnalyzedStatement::Continue]),
        });

        // While
        visitor.visit_statement(&AnalyzedStatement::While {
            condition: Box::new(dummy_expr.clone()),
            body: vec![AnalyzedStatement::Break],
        });

        // For
        visitor.visit_statement(&AnalyzedStatement::For {
            variable: "v".into(),
            iterator: Box::new(dummy_expr.clone()),
            body: vec![AnalyzedStatement::Break],
        });

        // Match
        visitor.visit_statement(&AnalyzedStatement::Match {
            scrutinee: Box::new(dummy_expr.clone()),
            arms: vec![(dummy_expr.clone(), vec![AnalyzedStatement::Break])],
        });

        // Return
        visitor.visit_statement(&AnalyzedStatement::Return {
            value: Some(Box::new(dummy_expr.clone())),
        });

        // FunctionDef
        visitor.visit_statement(&AnalyzedStatement::FunctionDef {
            name: "f".into(),
            params: vec![],
            return_type: None,
            body: vec![AnalyzedStatement::Break],
        });

        // TraitImplementation
        visitor.visit_statement(&AnalyzedStatement::TraitImplementation {
            trait_name: "T".into(),
            type_name: "X".into(),
            methods: vec![
                crate::semantic::AnalyzedMethod {
                    name: "m".into(),
                    params: vec![],
                    return_type: None,
                    body: Some(vec![AnalyzedStatement::Break]),
                }
            ],
        });

        // TestDeclaration
        visitor.visit_statement(&AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![AnalyzedStatement::Break],
        });

        // UnaryOp
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp {
                op: crate::morphology::lexicon::UnaryOp::Not,
                operand: Box::new(dummy_expr.clone()),
            },
            glossa_type: GlossaType::Boolean,
        });

        // BinOp
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp {
                left: Box::new(dummy_expr.clone()),
                op: crate::morphology::lexicon::BinaryOp::Add,
                right: Box::new(dummy_expr.clone()),
            },
            glossa_type: GlossaType::Number,
        });

        // PropertyAccess
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner: Box::new(dummy_expr.clone()),
                property: "prop".into(),
            },
            glossa_type: GlossaType::Number,
        });

        // MethodCall
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::MethodCall {
                receiver: Box::new(dummy_expr.clone()),
                method: "m".into(),
                args: vec![dummy_expr.clone()],
            },
            glossa_type: GlossaType::Number,
        });

        // FunctionCall
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::FunctionCall {
                func: "f".into(),
                args: vec![dummy_expr.clone()],
            },
            glossa_type: GlossaType::Number,
        });

        // StructInstantiation
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::StructInstantiation {
                type_name: "X".into(),
                fields: vec!["f".into()],
                args: vec![dummy_expr.clone()],
            },
            glossa_type: GlossaType::Number,
        });

        // ArrayLiteral
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(vec![dummy_expr.clone()]),
            glossa_type: GlossaType::Number,
        });

        // IndexAccess
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::IndexAccess {
                array: Box::new(dummy_expr.clone()),
                index: Box::new(dummy_expr.clone()),
            },
            glossa_type: GlossaType::Number,
        });

        // Lambda
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::Lambda {
                params: vec!["p".into()],
                body: Box::new(dummy_expr.clone()),
                capture_mode: crate::semantic::CaptureMode::Borrow,
            },
            glossa_type: GlossaType::Number,
        });

        // VerbCall
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::VerbCall {
                verb: "v".into(),
                args: vec![dummy_expr.clone()],
            },
            glossa_type: GlossaType::Number,
        });

        // Range
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::Range {
                start: Box::new(dummy_expr.clone()),
                end: Box::new(dummy_expr.clone()),
                inclusive: false,
            },
            glossa_type: GlossaType::Number,
        });

        // Try
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::Try(Box::new(dummy_expr.clone())),
            glossa_type: GlossaType::Number,
        });

        // Assert
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::Assert {
                condition: Box::new(dummy_expr.clone()),
            },
            glossa_type: GlossaType::Number,
        });

        // AssertEq
        visitor.visit_expr(&AnalyzedExpr {
            expr: AnalyzedExprKind::AssertEq {
                left: Box::new(dummy_expr.clone()),
                right: Box::new(dummy_expr.clone()),
            },
            glossa_type: GlossaType::Number,
        });
    }

    #[test]
    fn test_run_chronicle() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_chronicle.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all("ξ 1 ἔστω.".as_bytes()).unwrap();
        }

        let result = run_chronicle(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_chronicle_error() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_error.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"invalid syntax").unwrap();
        }

        let result = run_chronicle(&file_path);
        assert!(result.is_err());
    }
}
