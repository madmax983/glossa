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

#[derive(Default)]
pub struct GnomonVisitor {
    pub current_depth: usize,
    pub max_depth: usize,
}

impl GnomonVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        match stmt {
            AnalyzedStatement::While { body, .. } => {
                self.current_depth += 1;
                if self.current_depth > self.max_depth {
                    self.max_depth = self.current_depth;
                }
                for s in body {
                    self.visit_statement(s);
                }
                self.current_depth -= 1;
            }
            AnalyzedStatement::For { body, .. } => {
                self.current_depth += 1;
                if self.current_depth > self.max_depth {
                    self.max_depth = self.current_depth;
                }
                for s in body {
                    self.visit_statement(s);
                }
                self.current_depth -= 1;
            }
            AnalyzedStatement::If {
                then_body,
                else_body,
                ..
            } => {
                for s in then_body {
                    self.visit_statement(s);
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::Match { arms, .. } => {
                for (_, stmts) in arms {
                    for s in stmts {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                for s in body {
                    self.visit_statement(s);
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
}

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

    let mut visitor = GnomonVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
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

    let complexity = if visitor.max_depth == 0 {
        "O(1)".to_string()
    } else if visitor.max_depth == 1 {
        "O(N)".to_string()
    } else {
        format!("O(N^{})", visitor.max_depth)
    };

    table.add_row(vec![
        Cell::new("Max Loop Depth"),
        Cell::new(visitor.max_depth.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Estimated Big-O"),
        Cell::new(complexity).fg(if visitor.max_depth > 2 {
            Color::Red
        } else if visitor.max_depth == 2 {
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
        let mut visitor = GnomonVisitor::new();
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![],
        };
        visitor.visit_statement(&stmt);
        assert_eq!(visitor.max_depth, 1);
    }

    #[test]
    fn test_gnomon_for_loop() {
        let mut visitor = GnomonVisitor::new();
        let stmt = AnalyzedStatement::For {
            variable: SmolStr::new("x"),
            iterator: dummy_expr(),
            body: vec![],
        };
        visitor.visit_statement(&stmt);
        assert_eq!(visitor.max_depth, 1);
    }

    #[test]
    fn test_gnomon_nested_loops() {
        let mut visitor = GnomonVisitor::new();
        let inner_loop = AnalyzedStatement::For {
            variable: SmolStr::new("y"),
            iterator: dummy_expr(),
            body: vec![],
        };
        let outer_loop = AnalyzedStatement::While {
            condition: dummy_expr(),
            body: vec![inner_loop],
        };
        visitor.visit_statement(&outer_loop);
        assert_eq!(visitor.max_depth, 2);
    }
}
