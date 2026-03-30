//! The River (ὁ Ῥοῦς) - Control Flow Visualizer
//!
//! This module implements the "Flow" tool, generating Mermaid.js flowcharts
//! to visualize the execution paths of a ΓΛΩΣΣΑ program.
//!
//! # Philosophy: "Everything Flows" (Πάντα ῥεῖ)
//!
//! Heraclitus famously said "Everything flows and nothing stands still".
//! This tool visualizes the flow of computation through bindings, conditions,
//! loops, and functions, turning static code into a map of dynamic execution.

use crate::parser::parse;
use crate::semantic::{analyze_program, AnalyzedProgram, AnalyzedStatement};
use crate::tools::narrator::tell_expr;
use crate::tools::ui::Status;
use miette::Result;
use std::path::Path;

/// Run the River tool on a file
///
/// Reads the source file, compiles it to an AnalyzedProgram, and generates
/// a Mermaid flowchart representing the control flow.
pub fn run_flow(input: &Path) -> Result<()> {
    let source = crate::tools::runner::load_source(input)?;
    let status = Status::start_with_symbol("Ῥοῦς (The River)", "🌊");

    let ast = parse(&source)?;
    let program = analyze_program(&ast)?;
    let map = generate_flowchart(&program);

    status.success();

    println!();
    println!("```mermaid");
    println!("{}", map);
    println!("```");

    Ok(())
}

struct FlowBuilder {
    out: String,
    next_id: usize,
}

impl FlowBuilder {
    fn new() -> Self {
        Self {
            out: String::from("flowchart TD\n"),
            next_id: 1,
        }
    }

    fn get_id(&mut self) -> String {
        let id = format!("N{}", self.next_id);
        self.next_id += 1;
        id
    }

    fn add_node(&mut self, id: &str, label: &str, shape: NodeShape) {
        let (open, close) = match shape {
            NodeShape::Process => ("[", "]"),
            NodeShape::Decision => ("{", "}"),
            NodeShape::Terminal => ("([", "])"),
            NodeShape::Subroutine => ("[[", "]]"),
            NodeShape::Data => ("[(", ")]"),
        };
        // Escape quotes
        let safe_label = label.replace("\"", "'");
        self.out.push_str(&format!("    {}{}\"{}\"{}\n", id, open, safe_label, close));
    }

    fn add_edge(&mut self, from: &str, to: &str, label: Option<&str>) {
        if let Some(lbl) = label {
            self.out.push_str(&format!("    {} -->|{}| {}\n", from, lbl, to));
        } else {
            self.out.push_str(&format!("    {} --> {}\n", from, to));
        }
    }
}

enum NodeShape {
    Process,
    Decision,
    Terminal,
    Subroutine,
    Data,
}

/// Generates a Mermaid flowchart from an AnalyzedProgram
pub fn generate_flowchart(program: &AnalyzedProgram) -> String {
    let mut builder = FlowBuilder::new();

    let start_id = builder.get_id();
    builder.add_node(&start_id, "Start", NodeShape::Terminal);

    let tail_ids = process_statements(&program.statements, vec![start_id], &mut builder);

    let end_id = builder.get_id();
    builder.add_node(&end_id, "End", NodeShape::Terminal);

    for tail in tail_ids {
        builder.add_edge(&tail, &end_id, None);
    }

    builder.out
}

fn process_statements(
    stmts: &[AnalyzedStatement],
    mut parent_ids: Vec<String>,
    builder: &mut FlowBuilder,
) -> Vec<String> {
    for stmt in stmts {
        parent_ids = process_statement(stmt, &parent_ids, builder);
    }
    parent_ids
}

fn process_statement(
    stmt: &AnalyzedStatement,
    parent_ids: &[String],
    builder: &mut FlowBuilder,
) -> Vec<String> {
    if parent_ids.is_empty() {
        return vec![]; // Unreachable code
    }

    let node_id = builder.get_id();
    let mut tails = vec![node_id.clone()];

    match stmt {
        AnalyzedStatement::Binding { name, value, .. } => {
            let label = format!("Let {} = {}", name, tell_expr(value));
            builder.add_node(&node_id, &label, NodeShape::Data);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::Assignment { name, value } => {
            let label = format!("Set {} = {}", name, tell_expr(value));
            builder.add_node(&node_id, &label, NodeShape::Process);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::Print(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let label = format!("Print: {}", expr_strs.join(", "));
            builder.add_node(&node_id, &label, NodeShape::Subroutine);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let label = format!("Eval: {}", expr_strs.join(", "));
            builder.add_node(&node_id, &label, NodeShape::Process);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            let label = format!("Query: {}", expr_strs.join(", "));
            builder.add_node(&node_id, &label, NodeShape::Subroutine);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            let label = format!("If {}", tell_expr(condition));
            builder.add_node(&node_id, &label, NodeShape::Decision);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }

            let then_tails = process_statements(then_body, vec![node_id.clone()], builder);
            tails = then_tails;

            if let Some(else_stmts) = else_body {
                let else_tails = process_statements(else_stmts, vec![node_id.clone()], builder);
                tails.extend(else_tails);
            } else {
                tails.push(node_id.clone()); // The false branch goes straight to the end
            }
        }
        AnalyzedStatement::While { condition, body } => {
            let label = format!("While {}", tell_expr(condition));
            builder.add_node(&node_id, &label, NodeShape::Decision);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }

            let body_tails = process_statements(body, vec![node_id.clone()], builder);
            for bt in body_tails {
                builder.add_edge(&bt, &node_id, Some("Loop")); // Back edge
            }

            // The tail of a while loop is the false branch of the condition
            tails = vec![node_id.clone()];
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            let label = format!("For {} in {}", variable, tell_expr(iterator));
            builder.add_node(&node_id, &label, NodeShape::Decision);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }

            let body_tails = process_statements(body, vec![node_id.clone()], builder);
            for bt in body_tails {
                builder.add_edge(&bt, &node_id, Some("Next")); // Back edge
            }

            // The tail is exiting the loop
            tails = vec![node_id.clone()];
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            let label = format!("Match {}", tell_expr(scrutinee));
            builder.add_node(&node_id, &label, NodeShape::Decision);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }

            tails = vec![];
            for (pat, body) in arms {
                let pat_label = tell_expr(pat);

                let case_id = builder.get_id();
                builder.add_node(&case_id, &format!("Case {}", pat_label), NodeShape::Process);
                builder.add_edge(&node_id, &case_id, Some(&pat_label));

                let case_tails = process_statements(body, vec![case_id], builder);
                tails.extend(case_tails);
            }
        }
        AnalyzedStatement::Return { value } => {
            let label = match value {
                Some(v) => format!("Return {}", tell_expr(v)),
                None => "Return".to_string(),
            };
            builder.add_node(&node_id, &label, NodeShape::Terminal);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
            tails = vec![]; // Breaks control flow
        }
        AnalyzedStatement::Break => {
            builder.add_node(&node_id, "Break", NodeShape::Terminal);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
            // In a real implementation we would route this to the end of the loop,
            // but for a simple visualizer, we just end the local flow line.
            tails = vec![];
        }
        AnalyzedStatement::Continue => {
            builder.add_node(&node_id, "Continue", NodeShape::Terminal);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
            tails = vec![];
        }
        AnalyzedStatement::FunctionDef { name, .. } => {
            let label = format!("Define Function {}", name);
            builder.add_node(&node_id, &label, NodeShape::Subroutine);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
            // In a robust implementation, we would visualize the body separately.
            // For now, we just show it as a node.
        }
        AnalyzedStatement::TypeDefinition { name, .. } => {
            let label = format!("Define Type {}", name);
            builder.add_node(&node_id, &label, NodeShape::Data);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::TraitDefinition { name, .. } => {
            let label = format!("Define Trait {}", name);
            builder.add_node(&node_id, &label, NodeShape::Data);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::TraitImplementation { trait_name, type_name, .. } => {
            let label = format!("Impl {} for {}", trait_name, type_name);
            builder.add_node(&node_id, &label, NodeShape::Data);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
        AnalyzedStatement::TestDeclaration { name, .. } => {
            let label = format!("Test: {}", name);
            builder.add_node(&node_id, &label, NodeShape::Process);
            for p in parent_ids {
                builder.add_edge(p, &node_id, None);
            }
        }
    }

    tails
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_river_basic() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let map = generate_flowchart(&program);
        assert!(map.contains("flowchart TD"));
        assert!(map.contains("Start"));
        assert!(map.contains("Let ξ = 5"));
        assert!(map.contains("Print: `ξ`"));
        assert!(map.contains("End"));
        assert!(map.contains("-->"));
    }

    #[test]
    fn test_river_if_statement() {
        let source = "
            ξ 5 ἔστω.
            ἐὰν ξ 5 ἴσον ᾖ, «ναι» λέγε. εἰ δὲ μή, «οχι» λέγε.
        ";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let map = generate_flowchart(&program);
        assert!(map.contains("If (`ξ` Eq 5)"));
        assert!(map.contains("Print: 'ναι'"));
        assert!(map.contains("Print: 'οχι'"));
    }
}
