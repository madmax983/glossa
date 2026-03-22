//! The Chronicler (ὁ Χρονογράφος) - Execution Timeline Tracer
//!
//! This module implements an execution tracer that runs the analyzed AST
//! and records every state change, building a time-travel timeline.

use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use std::path::Path;

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, analyze_program};
use crate::tools::interpreter::{EvalError, Interpreter, Value};

/// Traces the execution of a file and prints the timeline.
pub fn run_trace(input: &Path) -> miette::Result<()> {
    let status =
        crate::tools::ui::Status::start_with_symbol("Χρονογραφία (Tracing execution)", "⏳");

    let source = crate::tools::runner::load_source(input)?;
    let ast = match parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            let err_msg = format!("Parse error: {}", e);
            status.error(&err_msg);
            return Err(miette::miette!("{}", err_msg));
        }
    };
    let program = match analyze_program(&ast) {
        Ok(p) => p,
        Err(e) => {
            let err_msg = format!("Semantic error: {}", e);
            status.error(&err_msg);
            return Err(miette::miette!("{}", err_msg));
        }
    };

    let mut tracer = TracingInterpreter::new();
    if let Err(e) = tracer.run(&program) {
        let err_msg = format!("Runtime error during trace: {}", e);
        status.error(&err_msg);
        return Err(miette::miette!("{}", err_msg));
    }

    status.success();
    tracer.print_timeline();

    Ok(())
}

/// A kind of event recorded in the execution trace.
#[derive(Debug, Clone)]
enum TraceEventKind {
    Binding { name: String, value: Value },
    Assignment { name: String, value: Value },
    Print { output: String },
}

/// A single step in the execution timeline.
#[derive(Debug, Clone)]
struct TraceEvent {
    step: usize,
    kind: TraceEventKind,
}

/// A wrapper around `Interpreter` that records state changes.
struct TracingInterpreter {
    interpreter: Interpreter,
    events: Vec<TraceEvent>,
    step_counter: usize,
}

impl TracingInterpreter {
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            events: Vec::new(),
            step_counter: 1,
        }
    }

    fn record_event(&mut self, kind: TraceEventKind) {
        self.events.push(TraceEvent {
            step: self.step_counter,
            kind,
        });
        self.step_counter += 1;
    }

    fn run(&mut self, program: &AnalyzedProgram) -> Result<(), EvalError> {
        for stmt in &program.statements {
            self.eval_statement_traced(stmt)?;
        }
        Ok(())
    }

    fn eval_statement_traced(&mut self, stmt: &AnalyzedStatement) -> Result<(), EvalError> {
        let before_output_len = self.interpreter.get_output().len();

        let wrap_prog = AnalyzedProgram {
            statements: vec![stmt.clone()],
            scope: crate::semantic::Scope::new(), // dummy scope for the single statement execution
        };

        // We run the single statement in the interpreter
        self.interpreter.run(&wrap_prog)?;

        let after_output_len = self.interpreter.get_output().len();

        // Check if anything was printed
        if after_output_len > before_output_len {
            let full_out = self.interpreter.get_output();
            let new_out = &full_out[before_output_len..];
            let lines: Vec<&str> = new_out.split('\n').filter(|s| !s.is_empty()).collect();
            for line in lines {
                self.record_event(TraceEventKind::Print {
                    output: line.to_string(),
                });
            }
        }

        // Check for bindings/assignments by looking at the statement AST
        match stmt {
            AnalyzedStatement::Binding { name, .. } => {
                let val_str = self.probe_variable(name)?;
                self.record_event(TraceEventKind::Binding {
                    name: name.to_string(),
                    value: Value::String(val_str), // Keep as string for display
                });
            }
            AnalyzedStatement::Assignment { name, .. } => {
                let val_str = self.probe_variable(name)?;
                self.record_event(TraceEventKind::Assignment {
                    name: name.to_string(),
                    value: Value::String(val_str),
                });
            }
            _ => {}
        }

        Ok(())
    }

    /// Probes a variable's value from the underlying Interpreter by running a temporary Print statement
    fn probe_variable(&mut self, name: &str) -> Result<String, EvalError> {
        let print_stmt = AnalyzedStatement::Print(vec![crate::semantic::AnalyzedExpr {
            expr: crate::semantic::AnalyzedExprKind::Variable(name.into()),
            glossa_type: crate::semantic::GlossaType::Unknown,
        }]);

        let wrap_prog = AnalyzedProgram {
            statements: vec![print_stmt],
            scope: crate::semantic::Scope::new(),
        };

        let before_len = self.interpreter.get_output().len();
        self.interpreter.run(&wrap_prog)?;
        let full_out = self.interpreter.get_output();

        // Strip the trailing newline from the newly printed part
        let new_part = full_out[before_len..]
            .trim_start_matches('\n')
            .trim_end_matches('\n');
        Ok(new_part.to_string())
    }

    fn print_timeline(&self) {
        println!();
        println!("   {}", "Γ Λ Ω Σ Σ Α   T R A C E R".bold().cyan());
        println!("   {}", "Execution Timeline Dashboard".italic().dim());
        println!();

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Step")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Action")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new("Variable")
                .add_attribute(Attribute::Bold)
                .fg(Color::Magenta),
            Cell::new("Value")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]);

        for event in &self.events {
            match &event.kind {
                TraceEventKind::Binding { name, value } => {
                    table.add_row(vec![
                        Cell::new(event.step.to_string()),
                        Cell::new("Binding").fg(Color::Blue),
                        Cell::new(name).fg(Color::Magenta),
                        Cell::new(value.to_string()).fg(Color::Green),
                    ]);
                }
                TraceEventKind::Assignment { name, value } => {
                    table.add_row(vec![
                        Cell::new(event.step.to_string()),
                        Cell::new("Assignment").fg(Color::DarkYellow),
                        Cell::new(name).fg(Color::Magenta),
                        Cell::new(value.to_string()).fg(Color::Green),
                    ]);
                }
                TraceEventKind::Print { output } => {
                    table.add_row(vec![
                        Cell::new(event.step.to_string()),
                        Cell::new("Print").fg(Color::Cyan),
                        Cell::new("-").fg(Color::DarkGrey),
                        Cell::new(output).fg(Color::White),
                    ]);
                }
            }
        }

        if self.events.is_empty() {
            table.add_row(vec![
                Cell::new("-"),
                Cell::new("No observable state changes"),
                Cell::new("-"),
                Cell::new("-"),
            ]);
        }

        println!("{table}");
        println!();
    }

    /// Returns the captured table as a string (useful for testing)
    #[allow(dead_code)]
    fn get_timeline_string(&self) -> String {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Step"),
            Cell::new("Action"),
            Cell::new("Variable"),
            Cell::new("Value"),
        ]);

        for event in &self.events {
            match &event.kind {
                TraceEventKind::Binding { name, value } => {
                    table.add_row(vec![
                        event.step.to_string(),
                        "Binding".to_string(),
                        name.to_string(),
                        value.to_string(),
                    ]);
                }
                TraceEventKind::Assignment { name, value } => {
                    table.add_row(vec![
                        event.step.to_string(),
                        "Assignment".to_string(),
                        name.to_string(),
                        value.to_string(),
                    ]);
                }
                TraceEventKind::Print { output } => {
                    table.add_row(vec![
                        event.step.to_string(),
                        "Print".to_string(),
                        "-".to_string(),
                        output.to_string(),
                    ]);
                }
            }
        }
        table.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_execution_timeline_output() {
        let source = "ξ πέντε ἔστω. μετά ξ πέντε ἔστω. ξ δέκα γίγνεται. ξ λέγε.";
        let ast = parse(source).expect("Failed to parse");
        let program = analyze_program(&ast).expect("Failed to analyze");

        let mut tracer = TracingInterpreter::new();
        tracer.run(&program).expect("Failed to trace");

        let timeline = tracer.get_timeline_string();

        assert!(timeline.contains("Binding"), "Missing binding event");
        assert!(timeline.contains("Assignment"), "Missing assignment event");
        assert!(timeline.contains("Print"), "Missing print event");

        // Verify exact values
        assert!(timeline.contains("ξ"), "Missing variable name");
        assert!(timeline.contains("5"), "Missing value 5");
        assert!(timeline.contains("10"), "Missing value 10");

        // Check step progression
        assert!(timeline.contains("1"), "Missing step 1");
        assert!(timeline.contains("2"), "Missing step 2");
        assert!(timeline.contains("3"), "Missing step 3");
    }

    #[test]
    fn test_trace_empty_events() {
        // Evaluate an expression without bindings or prints
        let source = "1 1 ἄθροισμα.";
        let ast = parse(source).expect("Failed to parse");
        let program = analyze_program(&ast).expect("Failed to analyze");

        let mut tracer = TracingInterpreter::new();
        tracer.run(&program).expect("Failed to trace");

        let timeline = tracer.get_timeline_string();
        // Since there are no events, the table will just have headers.
        assert!(!timeline.contains("Binding"));
        assert!(!timeline.contains("Print"));
    }

    #[test]
    fn test_run_trace_success() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("trace_success.γλ");
        std::fs::write(&file_path, "ξ 5 ἔστω.").unwrap();

        let result = run_trace(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_trace_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("trace_parse_error.γλ");
        std::fs::write(&file_path, "invalid syntax").unwrap();

        let result = run_trace(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_run_trace_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("trace_semantic_error.γλ");
        // We need an error that triggers during `analyze_program`.
        // Using `«test» 5 ἄθροισμα ἔστω.` is structurally sound (binding), but semantic analysis fails
        // because you can't add a string and a number.
        std::fs::write(&file_path, "«test» 5 ἄθροισμα ἔστω.").unwrap();

        let result = run_trace(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Semantic error"));
    }

    #[test]
    fn test_run_trace_runtime_error() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("trace_runtime_error.γλ");
        // Div by zero triggers an eval error at runtime
        std::fs::write(&file_path, "ξ 1 0 μέρος ἔστω.").unwrap();

        let result = run_trace(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Runtime error during trace"));
    }

    #[test]
    fn test_print_timeline_coverage() {
        let mut tracer = TracingInterpreter::new();
        // Empty events output
        tracer.print_timeline();

        // With events output
        tracer.record_event(TraceEventKind::Binding {
            name: "a".to_string(),
            value: Value::Number(1),
        });
        tracer.record_event(TraceEventKind::Assignment {
            name: "b".to_string(),
            value: Value::Number(2),
        });
        tracer.record_event(TraceEventKind::Print {
            output: "hello".to_string(),
        });
        tracer.print_timeline();
    }
}
