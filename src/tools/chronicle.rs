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
