//! The Labyrinth Tool ("Labyrinth")
//!
//! This module implements the "Labyrinth" functionality, which visualizes the
//! control flow of a ΓΛΩΣΣΑ program as a Mermaid.js flowchart.
//!
//! # Purpose
//!
//! It enables users to export their codebase into a structured flowchart,
//! helping visualize complex conditional and loop logic.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, analyze_program};
use crate::tools::narrator::tell_expr;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Run the Labyrinth tool on a file
///
/// Reads the source file, compiles it, and generates the flowchart to stdout.
pub fn run_labyrinth(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Λαβύρινθος (Labyrinth)", "🧭");

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

    let mut labyrinth_buffer = Vec::new();
    if let Err(e) = run_labyrinth_inner(&program, &mut labyrinth_buffer) {
        status.error("Σφάλμα (Error)");
        return Err(e);
    }
    let output = String::from_utf8(labyrinth_buffer).expect("outputs valid UTF-8");

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   L A B Y R I N T H".bold().cyan());
    println!("   {}", "Control Flow Graph".italic().dim());
    println!();
    println!("{}", output);

    Ok(())
}

/// Internal implementation of Labyrinth logic
///
/// Separated for testing purposes.
pub fn run_labyrinth_inner<W: std::io::Write>(
    program: &AnalyzedProgram,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "```mermaid").into_diagnostic()?;
    writeln!(writer, "graph TD").into_diagnostic()?;

    let mut builder = LabyrinthBuilder::new(writer);
    builder.build(program)?;

    writeln!(writer, "```").into_diagnostic()?;
    Ok(())
}

struct LabyrinthBuilder<'a, W: std::io::Write> {
    writer: &'a mut W,
    node_counter: usize,
}

impl<'a, W: std::io::Write> LabyrinthBuilder<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            node_counter: 0,
        }
    }

    fn next_node_id(&mut self) -> String {
        self.node_counter += 1;
        format!("node{}", self.node_counter)
    }

    fn build(&mut self, program: &AnalyzedProgram) -> Result<()> {
        let start_id = self.next_node_id();
        writeln!(self.writer, "    {}(([Start]))", start_id).into_diagnostic()?;

        let mut current_id = start_id;

        for stmt in &program.statements {
            current_id = self.visit_statement(stmt, &current_id, None)?;
        }

        let end_id = self.next_node_id();
        writeln!(self.writer, "    {}(([End]))", end_id).into_diagnostic()?;
        writeln!(self.writer, "    {} --> {}", current_id, end_id).into_diagnostic()?;

        Ok(())
    }

    fn write_edge(&mut self, from: &str, to: &str, label: Option<&str>) -> Result<()> {
        if let Some(lbl) = label {
            writeln!(self.writer, "    {} -- {} --> {}", from, lbl, to).into_diagnostic()?;
        } else {
            writeln!(self.writer, "    {} --> {}", from, to).into_diagnostic()?;
        }
        Ok(())
    }

    fn visit_statement(
        &mut self,
        stmt: &AnalyzedStatement,
        parent_id: &str,
        edge_label: Option<&str>,
    ) -> Result<String> {
        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                let id = self.next_node_id();
                let label = format!("let {} = {}", name, tell_expr(value));
                writeln!(self.writer, "    {}[{:?}]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::Assignment { name, value } => {
                let id = self.next_node_id();
                let label = format!("{} = {}", name, tell_expr(value));
                writeln!(self.writer, "    {}[{:?}]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::Print(exprs) => {
                let id = self.next_node_id();
                let args = exprs
                    .iter()
                    .map(|e| tell_expr(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                let label = format!("Print({})", args);
                writeln!(self.writer, "    {}[/{:?}/]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::Query(exprs) => {
                let id = self.next_node_id();
                let args = exprs
                    .iter()
                    .map(|e| tell_expr(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                let label = format!("Query({})", args);
                writeln!(self.writer, "    {}[/{:?}/]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::Expression(exprs) => {
                let id = self.next_node_id();
                let args = exprs
                    .iter()
                    .map(|e| tell_expr(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                let label = format!("Eval({})", args);
                writeln!(self.writer, "    {}[{:?}]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_id = self.next_node_id();
                let cond_label = format!("if {}", tell_expr(condition));
                writeln!(self.writer, "    {}{{{:?}}}", cond_id, cond_label).into_diagnostic()?;
                self.write_edge(parent_id, &cond_id, edge_label)?;

                // Then branch
                let mut then_end_id = cond_id.clone();
                let mut first_then = true;
                for then_stmt in then_body {
                    let next_label = if first_then { Some("Yes") } else { None };
                    let next_id = self.visit_statement(then_stmt, &then_end_id, next_label)?;
                    then_end_id = next_id;
                    first_then = false;
                }

                let merge_id = self.next_node_id();
                writeln!(self.writer, "    {}(( ))", merge_id).into_diagnostic()?;

                // Else branch
                let mut else_end_id = cond_id.clone();
                if let Some(else_stmts) = else_body {
                    let mut first_else = true;
                    for else_stmt in else_stmts {
                        let next_label = if first_else { Some("No") } else { None };
                        let next_id = self.visit_statement(else_stmt, &else_end_id, next_label)?;
                        else_end_id = next_id;
                        first_else = false;
                    }
                    self.write_edge(&else_end_id, &merge_id, None)?;
                } else {
                    // Route "No" branch directly to merge node instead of an empty node
                    self.write_edge(&cond_id, &merge_id, Some("No"))?;
                }

                if first_then {
                    // If then block was empty
                    self.write_edge(&cond_id, &merge_id, Some("Yes"))?;
                } else {
                    self.write_edge(&then_end_id, &merge_id, None)?;
                }

                Ok(merge_id)
            }
            AnalyzedStatement::While { condition, body } => {
                let cond_id = self.next_node_id();
                let cond_label = format!("while {}", tell_expr(condition));
                writeln!(self.writer, "    {}{{{:?}}}", cond_id, cond_label).into_diagnostic()?;
                self.write_edge(parent_id, &cond_id, edge_label)?;

                let mut body_end_id = cond_id.clone();
                let mut first_body = true;
                for body_stmt in body {
                    let next_label = if first_body { Some("Yes") } else { None };
                    let next_id = self.visit_statement(body_stmt, &body_end_id, next_label)?;
                    body_end_id = next_id;
                    first_body = false;
                }

                if first_body {
                    // Empty body loops back immediately
                    self.write_edge(&cond_id, &cond_id, Some("Yes"))?;
                } else {
                    // Loop back to condition
                    self.write_edge(&body_end_id, &cond_id, None)?;
                }

                let after_id = self.next_node_id();
                writeln!(self.writer, "    {}(( ))", after_id).into_diagnostic()?;
                self.write_edge(&cond_id, &after_id, Some("No"))?;

                Ok(after_id)
            }
            AnalyzedStatement::For { variable, iterator, body } => {
                let cond_id = self.next_node_id();
                let cond_label = format!("for {} in {}", variable, tell_expr(iterator));
                writeln!(self.writer, "    {}{{{:?}}}", cond_id, cond_label).into_diagnostic()?;
                self.write_edge(parent_id, &cond_id, edge_label)?;

                let mut body_end_id = cond_id.clone();
                let mut first_body = true;
                for body_stmt in body {
                    let next_label = if first_body { Some("Next") } else { None };
                    let next_id = self.visit_statement(body_stmt, &body_end_id, next_label)?;
                    body_end_id = next_id;
                    first_body = false;
                }

                if first_body {
                    self.write_edge(&cond_id, &cond_id, Some("Next"))?;
                } else {
                    self.write_edge(&body_end_id, &cond_id, None)?;
                }

                let after_id = self.next_node_id();
                writeln!(self.writer, "    {}(( ))", after_id).into_diagnostic()?;
                self.write_edge(&cond_id, &after_id, Some("Done"))?;

                Ok(after_id)
            }
            AnalyzedStatement::Break => {
                let id = self.next_node_id();
                writeln!(self.writer, "    {}[Break]", id).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::Continue => {
                let id = self.next_node_id();
                writeln!(self.writer, "    {}[Continue]", id).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            AnalyzedStatement::Return { value } => {
                let id = self.next_node_id();
                let label = if let Some(v) = value {
                    format!("Return({})", tell_expr(v))
                } else {
                    "Return".to_string()
                };
                writeln!(self.writer, "    {}[{:?}]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
            _ => {
                // Catch-all for defs and unhandled variants
                let id = self.next_node_id();
                let label = match stmt {
                    AnalyzedStatement::FunctionDef { name, .. } => format!("FunctionDef({})", name),
                    AnalyzedStatement::TypeDefinition { name, .. } => format!("TypeDefinition({})", name),
                    AnalyzedStatement::TraitDefinition { name, .. } => format!("TraitDefinition({})", name),
                    AnalyzedStatement::TraitImplementation { type_name, trait_name, .. } => format!("Impl {} for {}", trait_name, type_name),
                    AnalyzedStatement::TestDeclaration { name, .. } => format!("Test({})", name),
                    AnalyzedStatement::Match { .. } => "Match".to_string(), // Match logic is complex to graph cleanly, so we simplify
                    _ => "Unknown Statement".to_string(),
                };
                writeln!(self.writer, "    {}[{:?}]", id, label).into_diagnostic()?;
                self.write_edge(parent_id, &id, edge_label)?;
                Ok(id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labyrinth_if_statement() {
        // We write an expression that works with the analyzer
        let source = "εἰ 1 > 0, «yes» λέγε.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let mut buffer = Vec::new();
        run_labyrinth_inner(&program, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // The expected mermaid chart should have a condition node {1 > 0} and a path to Print.
        assert!(output.contains("{if (1 > 0)}"));
        assert!(output.contains("Print"));
    }
}
