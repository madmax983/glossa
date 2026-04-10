//! The Chronomancer Tool ("Trace")
//!
//! This module implements the "Trace" functionality, an interactive time-traveling
//! debugger that steps through code execution and records variable mutations.
//!
//! # Purpose
//!
//! "Trace" allows users to visualize the state of variables across the execution of a
//! ΓΛΩΣΣΑ program.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, analyze_program};
use crate::tools::interpreter::Interpreter;
use crate::tools::narrator::tell_expr;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Cell, Color, Table};
use miette::Result;
use std::collections::HashMap;
use std::path::Path;

/// Run the Chronomancer tool on a file
pub fn run_chronomancer(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Χρονομάντης (Tracing)", "⏳");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    // 1. Parse & Analyze
    let ast = match parse(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα συντάξεως (Syntax Error)");
            return Err(miette::miette!("{}", e));
        }
    };
    let program = match analyze_program(&ast) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα σημασίας (Semantic Error)");
            return Err(miette::miette!("{}", e));
        }
    };

    // 2. Trace Execution
    let mut tracer = Tracer::new();
    tracer.trace(&program);

    status.success();

    // 3. Display Trace Table
    println!();
    tracer.print_trace_table();

    Ok(())
}

/// A lightweight tracer that steps through the program and records variable states.
pub struct Tracer {
    interpreter: Interpreter,
    history: Vec<TraceStep>,
}

struct TraceStep {
    statement_desc: String,
    state: HashMap<String, String>,
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracer {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            history: Vec::new(),
        }
    }

    pub fn trace(&mut self, program: &AnalyzedProgram) {
        // Record initial state
        self.record_state("Program Start");

        for stmt in &program.statements {
            let desc = self.describe_statement(stmt);

            // Execute the statement
            let _ = self.interpreter.eval_statement(stmt);

            // Record state after execution
            self.record_state(&desc);
        }
    }

    fn record_state(&mut self, desc: &str) {
        let mut current_state = HashMap::new();

        for scope in &self.interpreter.env {
            for (k, v) in scope {
                current_state.insert(k.clone(), v.to_string());
            }
        }

        self.history.push(TraceStep {
            statement_desc: desc.to_string(),
            state: current_state,
        });
    }

    fn describe_statement(&self, stmt: &AnalyzedStatement) -> String {
        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                format!("Let {} = {}", name, tell_expr(value))
            }
            AnalyzedStatement::Print(exprs) => {
                let s: Vec<String> = exprs.iter().map(tell_expr).collect();
                format!("Print {}", s.join(", "))
            }
            AnalyzedStatement::Expression(exprs) => {
                let s: Vec<String> = exprs.iter().map(tell_expr).collect();
                format!("Eval {}", s.join(", "))
            }
            AnalyzedStatement::If { condition, .. } => {
                format!("If {}", tell_expr(condition))
            }
            _ => "Other Statement".to_string(),
        }
    }

    pub fn print_trace_table(&self) {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Step").fg(Color::Cyan),
            Cell::new("Action").fg(Color::Yellow),
            Cell::new("Variable State").fg(Color::Green),
        ]);

        for (i, step) in self.history.iter().enumerate() {
            let mut state_str = String::new();
            if step.state.is_empty() {
                state_str.push_str("<empty>");
            } else {
                for (k, v) in &step.state {
                    state_str.push_str(&format!("{}: {}\n", k, v));
                }
            }

            table.add_row(vec![
                Cell::new(i.to_string()),
                Cell::new(&step.statement_desc),
                Cell::new(state_str.trim_end()),
            ]);
        }

        println!("{table}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};

    #[test]
    fn test_tracer_dummy_ast() {
        let scope = Scope::new();

        let stmt1 = AnalyzedStatement::Binding {
            name: "x".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(42),
                glossa_type: GlossaType::Number,
            },
            mutable: false,
        };

        let stmt2 = AnalyzedStatement::Print(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Number,
        }]);

        let program = AnalyzedProgram {
            statements: vec![stmt1, stmt2],
            scope,
        };

        let mut tracer = Tracer::new();
        tracer.trace(&program);

        // history should have:
        // 0. Program Start
        // 1. Let x = 42
        // 2. Print `x`
        assert_eq!(tracer.history.len(), 3);
        assert_eq!(tracer.history[0].statement_desc, "Program Start");
        assert!(tracer.history[1].statement_desc.contains("Let x = 42"));
        assert!(tracer.history[2].statement_desc.contains("Print `x`"));

        // At step 1 (after Binding), state should have "x": "42"
        assert_eq!(tracer.history[1].state.get("x").unwrap(), "42");
    }
}
