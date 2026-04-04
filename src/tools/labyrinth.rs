//! The Labyrinth Tool ("Labyrinth")
//!
//! This module implements a Control Flow Graph (CFG) generator using Mermaid.js.
//! It traverses the `AnalyzedProgram` and creates a graph showing the execution paths,
//! particularly highlighting branching logic like `If` and `While` statements.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement};
use crate::tools::narrator::tell_expr;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::Path;

/// Run the Labyrinth tool to generate a CFG
pub fn run_labyrinth(input: &Path) -> Result<()> {
    use crate::parser::parse;
    use crate::semantic::analyze_program;
    use crate::tools::runner::load_source;

    let status = Status::start_with_symbol("Λαβύρινθος (Labyrinth)", "🪢");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

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

    let cfg_mermaid = generate_cfg(&program);

    let output_path = input.with_extension("cfg.md");
    let md = format!(
        "# Control Flow Graph: `{}`\n\n```mermaid\n{}\n```\n",
        input.file_name().unwrap_or_default().to_string_lossy(),
        cfg_mermaid
    );

    if let Err(e) = fs::write(&output_path, &md).into_diagnostic() {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(e);
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   L A B Y R I N T H".bold().cyan());
    println!("   {}", "Control Flow Graph Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

/// Generates a Mermaid.js `graph TD` string representing the CFG
pub fn generate_cfg(program: &AnalyzedProgram) -> String {
    let mut buffer = Vec::new();
    buffer.push("graph TD".to_string());

    let mut next_id = 0;
    build_statements(&program.statements, &mut buffer, &mut next_id, None, None);

    buffer.join("\n")
}

/// Recursively builds the statements into the Mermaid buffer
fn build_statements(
    statements: &[AnalyzedStatement],
    buffer: &mut Vec<String>,
    next_id: &mut usize,
    mut edge_label: Option<&str>,
    parent_node: Option<usize>,
) -> Option<usize> {
    let mut last_node = parent_node;
    for stmt in statements {
        let current_node = *next_id;
        *next_id += 1;

        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                buffer.push(format!(
                    "N{}[\"Let `{}` be {}\"]",
                    current_node,
                    name,
                    tell_expr(value)
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Assignment { name, value } => {
                buffer.push(format!(
                    "N{}[\"`{}` becomes {}\"]",
                    current_node,
                    name,
                    tell_expr(value)
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Print(exprs) => {
                let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
                buffer.push(format!(
                    "N{}[\"Print {}\"]",
                    current_node,
                    expr_strs.join(", ")
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Expression(exprs) => {
                let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
                buffer.push(format!(
                    "N{}[\"Expression: {}\"]",
                    current_node,
                    expr_strs.join(", ")
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Query(exprs) => {
                let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
                buffer.push(format!(
                    "N{}[\"Query: {}\"]",
                    current_node,
                    expr_strs.join(", ")
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                buffer.push(format!(
                    "N{}{{\"if {}\"}}",
                    current_node,
                    tell_expr(condition)
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);

                let _then_last =
                    build_statements(then_body, buffer, next_id, Some("Yes"), last_node);
                if let Some(else_branch) = else_body {
                    let _else_last =
                        build_statements(else_branch, buffer, next_id, Some("No"), last_node);
                } else {
                    let _else_last = last_node;
                }

                // For simplicity in this dummy representation, we don't merge branches back.
                // In a real CFG you'd have an end node to join paths.
            }
            AnalyzedStatement::While { condition, body } => {
                buffer.push(format!(
                    "N{}{{\"while {}\"}}",
                    current_node,
                    tell_expr(condition)
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);

                let loop_last = build_statements(body, buffer, next_id, Some("Loop"), last_node);

                if let Some(ll) = loop_last {
                    buffer.push(format!("N{} --> N{}", ll, current_node));
                }
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                buffer.push(format!(
                    "N{}{{\"for {} in {}\"}}",
                    current_node,
                    variable,
                    tell_expr(iterator)
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
                let loop_last = build_statements(body, buffer, next_id, Some("Next"), last_node);
                if let Some(ll) = loop_last {
                    buffer.push(format!("N{} --> N{}", ll, current_node));
                }
            }
            AnalyzedStatement::Return { value } => {
                let v_str = value
                    .as_ref()
                    .map(|v| tell_expr(v))
                    .unwrap_or_else(|| "nothing".to_string());
                buffer.push(format!("N{}[\"Return {}\"]", current_node, v_str));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Break => {
                buffer.push(format!("N{}[\"Break\"]", current_node));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Continue => {
                buffer.push(format!("N{}[\"Continue\"]", current_node));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::FunctionDef { name, .. } => {
                buffer.push(format!("N{}[\"Function {}\"]", current_node, name));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::TypeDefinition { name, .. } => {
                buffer.push(format!("N{}[\"Type {}\"]", current_node, name));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::TraitDefinition { name, .. } => {
                buffer.push(format!("N{}[\"Trait {}\"]", current_node, name));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::TraitImplementation {
                type_name,
                trait_name,
                ..
            } => {
                buffer.push(format!(
                    "N{}[\"Impl {} for {}\"]",
                    current_node, trait_name, type_name
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::TestDeclaration { name, .. } => {
                buffer.push(format!("N{}[\"Test {}\"]", current_node, name));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                buffer.push(format!(
                    "N{}{{\"match {}\"}}",
                    current_node,
                    tell_expr(scrutinee)
                ));
                if let Some(p) = last_node {
                    let label = edge_label
                        .take()
                        .map(|l| format!(" -- \"{}\" --> ", l))
                        .unwrap_or_else(|| " --> ".to_string());
                    buffer.push(format!("N{}{}N{}", p, label, current_node));
                }
                last_node = Some(current_node);

                for (pattern, body) in arms {
                    let arm_label = tell_expr(pattern);
                    let _arm_last =
                        build_statements(body, buffer, next_id, Some(&arm_label), last_node);
                }
            }
        }
    }
    last_node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
    use smol_str::SmolStr;

    #[test]
    fn test_labyrinth_generate_cfg() {
        let scope = Scope::new();
        let program = AnalyzedProgram {
            statements: vec![
                AnalyzedStatement::Binding {
                    name: SmolStr::new("x"),
                    value: AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    },
                    mutable: true,
                },
                AnalyzedStatement::If {
                    condition: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::BooleanLiteral(true),
                        glossa_type: GlossaType::Boolean,
                    }),
                    then_body: vec![AnalyzedStatement::Expression(vec![AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(2),
                        glossa_type: GlossaType::Number,
                    }])],
                    else_body: Some(vec![AnalyzedStatement::Expression(vec![AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(3),
                        glossa_type: GlossaType::Number,
                    }])]),
                },
            ],
            scope,
        };

        let cfg = generate_cfg(&program);

        assert!(cfg.contains("graph TD"));
        assert!(cfg.contains("N0[\"Let `x` be 1\"]"));
        assert!(cfg.contains("N1{\"if true\"}"));
        assert!(cfg.contains("N0 --> N1"));
        assert!(cfg.contains("N1 -- \"Yes\" --> N2"));
        assert!(cfg.contains("N1 -- \"No\" --> N3"));
    }
}
