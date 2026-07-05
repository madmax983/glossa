//! The Philosopher (ὁ Φιλόσοφος) - Functional Purity & Stoicism Scorer
//!
//! This module implements the "Philosopher" tool, which traverses the ΓΛΩΣΣΑ
//! program's semantic AST and evaluates its "purity" based on functional
//! programming principles. Programs with fewer side effects and no mutable
//! state are considered more "Stoic".

use crate::semantic::AnalyzedStatement;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// A visitor that traverses the AST to calculate functional purity metrics.
#[derive(Default)]
pub struct PhilosopherVisitor {
    /// Number of immutable bindings (`ἔστω`)
    pub immutable_bindings: usize,
    /// Number of mutable bindings (`μετὰ ... ἔστω`)
    pub mutable_bindings: usize,
    /// Number of re-assignments (`γίγνεται`)
    pub assignments: usize,
    /// Number of side-effect expressions or print statements
    pub side_effects: usize,
    /// Total number of statements visited
    pub total_statements: usize,
}

impl PhilosopherVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        self.total_statements += 1;
        match stmt {
            AnalyzedStatement::Binding { mutable, .. } => {
                if *mutable {
                    self.mutable_bindings += 1;
                } else {
                    self.immutable_bindings += 1;
                }
            }
            AnalyzedStatement::Assignment { .. } => {
                self.assignments += 1;
            }
            AnalyzedStatement::Print(_) | AnalyzedStatement::Expression(_) => {
                self.side_effects += 1;
            }
            AnalyzedStatement::Query(_) => {
                self.side_effects += 1;
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
            AnalyzedStatement::While { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::For { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::Match { arms, .. } => {
                for (_, arm_body) in arms {
                    for s in arm_body {
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

    pub fn calculate_score(&self) -> u32 {
        if self.total_statements == 0 {
            return 100;
        }

        let penalty =
            (self.mutable_bindings * 10) + (self.assignments * 15) + (self.side_effects * 5);

        let score = 100i32.saturating_sub(penalty as i32);
        score.max(0) as u32
    }
}

pub fn run_philosopher(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Φιλόσοφος (Philosopher Analysis)", "🦉");

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
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    status.success();

    let mut visitor = PhilosopherVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }

    let score = visitor.calculate_score();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   P H I L O S O P H E R".cyan().bold());
    println!("   {}", "Stoicism & Purity Report".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Metric")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);

    table.add_row(vec![
        Cell::new("Immutable Bindings"),
        Cell::new(visitor.immutable_bindings.to_string()).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Mutable Bindings"),
        Cell::new(visitor.mutable_bindings.to_string()).fg(if visitor.mutable_bindings > 0 {
            Color::Red
        } else {
            Color::White
        }),
    ]);
    table.add_row(vec![
        Cell::new("Assignments"),
        Cell::new(visitor.assignments.to_string()).fg(if visitor.assignments > 0 {
            Color::Red
        } else {
            Color::White
        }),
    ]);
    table.add_row(vec![
        Cell::new("Side Effects"),
        Cell::new(visitor.side_effects.to_string()).fg(Color::Yellow),
    ]);

    let score_color = match score {
        90..=100 => Color::Green,
        50..=89 => Color::Yellow,
        _ => Color::Red,
    };

    table.add_row(vec![
        Cell::new("Stoicism Score"),
        Cell::new(format!("{}%", score))
            .fg(score_color)
            .add_attribute(Attribute::Bold),
    ]);

    println!("{table}");
    println!();

    let quote = match score {
        100 => "«Οὐδὲν ἁμαρτάνειν ἐστὶ θεῶν» (To make no mistakes is for the gods - you are pure!)",
        80..=99 => "«Μηδὲν ἄγαν» (Nothing in excess - well balanced)",
        50..=79 => "«Πάντα ῥεῖ» (Everything flows - you embrace mutation)",
        _ => "«Ἄνθρωπος μέτρον πάντων» (Man is the measure of all things - pure chaos!)",
    };

    println!("   {} {}", "Verdict:".bold().magenta(), quote.italic());
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};
    use smol_str::SmolStr;

    fn dummy_expr() -> AnalyzedExpr {
        AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        }
    }

    #[test]
    fn test_philosopher_visitor_pure() {
        let mut visitor = PhilosopherVisitor::new();
        visitor.visit_statement(&AnalyzedStatement::Binding {
            name: SmolStr::new("x"),
            value: dummy_expr(),
            mutable: false,
        });

        assert_eq!(visitor.immutable_bindings, 1);
        assert_eq!(visitor.mutable_bindings, 0);
        assert_eq!(visitor.calculate_score(), 100);
    }

    #[test]
    fn test_philosopher_visitor_impure() {
        let mut visitor = PhilosopherVisitor::new();
        visitor.visit_statement(&AnalyzedStatement::Binding {
            name: SmolStr::new("x"),
            value: dummy_expr(),
            mutable: true,
        });
        visitor.visit_statement(&AnalyzedStatement::Assignment {
            name: SmolStr::new("x"),
            value: dummy_expr(),
        });
        visitor.visit_statement(&AnalyzedStatement::Print(vec![]));

        assert_eq!(visitor.mutable_bindings, 1);
        assert_eq!(visitor.assignments, 1);
        assert_eq!(visitor.side_effects, 1);
        // Penalty: (1*10) + (1*15) + (1*5) = 30
        assert_eq!(visitor.calculate_score(), 70);
    }
}
