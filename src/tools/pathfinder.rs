//! The Labyrinth Tool ("Pathfinder")
//!
//! This module implements a tool that generates a Mermaid.js control flow graph (CFG)
//! from the semantic AST (`AnalyzedProgram`). It visualizes the execution paths,
//! branching logic, and loops of a ΓΛΩΣΣΑ program.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, analyze_program};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use miette::Result;
use std::path::Path;

/// Run the Pathfinder tool on a file to generate a control flow graph
pub fn run_pathfinder(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Λαβύρινθος (Pathfinder)", "🗺️");

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

    let graph = generate_cfg(&program);

    status.success();

    println!();
    println!("```mermaid");
    println!("{}", graph);
    println!("```");
    println!();

    Ok(())
}

struct CfgContext {
    node_counter: usize,
    edges: Vec<String>,
    nodes: Vec<String>,
}

impl CfgContext {
    fn new() -> Self {
        Self {
            node_counter: 0,
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn next_node_id(&mut self) -> String {
        let id = format!("N{}", self.node_counter);
        self.node_counter += 1;
        id
    }

    fn add_node(&mut self, id: &str, label: &str, shape: &str) {
        let (open, close) = match shape {
            "rect" => ("[", "]"),
            "rounded" => ("(", ")"),
            "diamond" => ("{", "}"),
            "circle" => ("((", "))"),
            "hexagon" => ("{{", "}}"),
            _ => ("[", "]"),
        };
        // Sanitize label to avoid mermaid parsing issues
        let safe_label = label.replace("\"", "'").replace("\n", " ");
        self.nodes
            .push(format!("    {}{}{}{}{}", id, open, "\"", safe_label, "\"").to_string() + close);
    }

    fn add_edge(&mut self, from: &str, to: &str, label: Option<&str>) {
        if let Some(l) = label {
            self.edges
                .push(format!("    {}-- \"{}\" -->{}", from, l, to));
        } else {
            self.edges.push(format!("    {}-->{}", from, to));
        }
    }
}

pub fn generate_cfg(program: &AnalyzedProgram) -> String {
    let mut ctx = CfgContext::new();

    let start_id = ctx.next_node_id();
    ctx.add_node(&start_id, "Start", "circle");

    let end_id = ctx.next_node_id();
    ctx.add_node(&end_id, "End", "circle");

    if program.statements.is_empty() {
        ctx.add_edge(&start_id, &end_id, None);
    } else {
        let last_node = traverse_block(&program.statements, &mut ctx, &start_id);
        ctx.add_edge(&last_node, &end_id, None);
    }

    let mut output = String::new();
    output.push_str("graph TD\n");
    for node in &ctx.nodes {
        output.push_str(node);
        output.push('\n');
    }
    for edge in &ctx.edges {
        output.push_str(edge);
        output.push('\n');
    }

    output
}

/// Traverses a block of statements and returns the ID of the last node in the flow,
/// hooking it up to the predecessor node.
fn traverse_block(
    statements: &[AnalyzedStatement],
    ctx: &mut CfgContext,
    predecessor: &str,
) -> String {
    let mut current_pred = predecessor.to_string();

    for stmt in statements {
        let next_node = traverse_statement(stmt, ctx, &current_pred);
        current_pred = next_node;
    }

    current_pred
}

fn traverse_statement(stmt: &AnalyzedStatement, ctx: &mut CfgContext, predecessor: &str) -> String {
    match stmt {
        AnalyzedStatement::Binding { name, .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, &format!("Let {} = ...", name), "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::Assignment { name, .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, &format!("Assign {} = ...", name), "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::Print(_) => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, "Print", "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::Expression(_) => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, "Expression", "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::Query(_) => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, "Query", "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::If {
            condition: _,
            then_body,
            else_body,
        } => {
            let cond_id = ctx.next_node_id();
            ctx.add_node(&cond_id, "If Condition", "diamond");
            ctx.add_edge(predecessor, &cond_id, None);

            let merge_id = ctx.next_node_id();
            ctx.add_node(&merge_id, "Merge", "circle");

            // Then branch
            let dummy_then = ctx.next_node_id();
            ctx.add_node(&dummy_then, "Then", "circle");
            ctx.add_edge(&cond_id, &dummy_then, Some("Yes"));

            let last_then = if then_body.is_empty() {
                dummy_then
            } else {
                traverse_block(then_body, ctx, &dummy_then)
            };
            ctx.add_edge(&last_then, &merge_id, None);

            // Else branch
            let dummy_else = ctx.next_node_id();
            ctx.add_node(&dummy_else, "Else", "circle");
            ctx.add_edge(&cond_id, &dummy_else, Some("No"));

            let last_else = if let Some(body) = else_body {
                if body.is_empty() {
                    dummy_else
                } else {
                    traverse_block(body, ctx, &dummy_else)
                }
            } else {
                dummy_else
            };
            ctx.add_edge(&last_else, &merge_id, None);

            merge_id
        }
        AnalyzedStatement::While { condition: _, body } => {
            let cond_id = ctx.next_node_id();
            ctx.add_node(&cond_id, "While Condition", "diamond");
            ctx.add_edge(predecessor, &cond_id, None);

            let loop_end_id = ctx.next_node_id();
            ctx.add_node(&loop_end_id, "End While", "circle");

            // Loop body
            let dummy_body = ctx.next_node_id();
            ctx.add_node(&dummy_body, "Do", "circle");
            ctx.add_edge(&cond_id, &dummy_body, Some("Yes"));

            let last_body = if body.is_empty() {
                dummy_body
            } else {
                traverse_block(body, ctx, &dummy_body)
            };

            // Back edge
            ctx.add_edge(&last_body, &cond_id, None);

            // Exit edge
            ctx.add_edge(&cond_id, &loop_end_id, Some("No"));

            loop_end_id
        }
        AnalyzedStatement::For { variable, body, .. } => {
            let cond_id = ctx.next_node_id();
            ctx.add_node(&cond_id, &format!("For {} in ...", variable), "hexagon");
            ctx.add_edge(predecessor, &cond_id, None);

            let loop_end_id = ctx.next_node_id();
            ctx.add_node(&loop_end_id, "End For", "circle");

            // Loop body
            let dummy_body = ctx.next_node_id();
            ctx.add_node(&dummy_body, "Do", "circle");
            ctx.add_edge(&cond_id, &dummy_body, Some("Next"));

            let last_body = if body.is_empty() {
                dummy_body
            } else {
                traverse_block(body, ctx, &dummy_body)
            };

            // Back edge
            ctx.add_edge(&last_body, &cond_id, None);

            // Exit edge
            ctx.add_edge(&cond_id, &loop_end_id, Some("Done"));

            loop_end_id
        }
        AnalyzedStatement::Match { arms, .. } => {
            let match_id = ctx.next_node_id();
            ctx.add_node(&match_id, "Match", "diamond");
            ctx.add_edge(predecessor, &match_id, None);

            let merge_id = ctx.next_node_id();
            ctx.add_node(&merge_id, "Merge", "circle");

            for (i, (_, arm_body)) in arms.iter().enumerate() {
                let dummy_arm = ctx.next_node_id();
                ctx.add_node(&dummy_arm, &format!("Arm {}", i), "circle");
                ctx.add_edge(&match_id, &dummy_arm, Some(&format!("Case {}", i)));

                let last_arm = if arm_body.is_empty() {
                    dummy_arm
                } else {
                    traverse_block(arm_body, ctx, &dummy_arm)
                };

                ctx.add_edge(&last_arm, &merge_id, None);
            }

            if arms.is_empty() {
                ctx.add_edge(&match_id, &merge_id, None);
            }

            merge_id
        }
        AnalyzedStatement::Break => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, "Break", "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::Continue => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, "Continue", "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::Return { .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, "Return", "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::FunctionDef { name, .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, &format!("Fn {}", name), "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::TypeDefinition { name, .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, &format!("Type {}", name), "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::TraitDefinition { name, .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, &format!("Trait {}", name), "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            ..
        } => {
            let id = ctx.next_node_id();
            ctx.add_node(
                &id,
                &format!("Impl {} for {}", trait_name, type_name),
                "rect",
            );
            ctx.add_edge(predecessor, &id, None);
            id
        }
        AnalyzedStatement::TestDeclaration { name, .. } => {
            let id = ctx.next_node_id();
            ctx.add_node(&id, &format!("Test {}", name), "rect");
            ctx.add_edge(predecessor, &id, None);
            id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_generate_cfg_basic() {
        let code = "ξ πέντε ἔστω. ξ λέγε.";
        let ast = parse(code).unwrap();
        let program = analyze_program(&ast).unwrap();

        let graph = generate_cfg(&program);
        assert!(graph.contains("graph TD"));
        assert!(graph.contains("Let ξ = ..."));
        assert!(graph.contains("Print"));
    }

    #[test]
    fn test_generate_cfg_if() {
        let code = "εἰ ἀληθές ἐστι, «ναι» λέγε.";
        let ast = parse(code).unwrap();
        let program = analyze_program(&ast).unwrap();

        let graph = generate_cfg(&program);
        assert!(graph.contains("If Condition"));
        assert!(graph.contains("-- \"Yes\" -->"));
        assert!(graph.contains("-- \"No\" -->"));
    }

    #[test]
    fn test_generate_cfg_while() {
        let code = "ἕως ἀληθές ἐστι, παῦε.";
        let ast = parse(code).unwrap();
        let program = analyze_program(&ast).unwrap();

        let graph = generate_cfg(&program);
        assert!(graph.contains("While Condition"));
        assert!(graph.contains("-- \"Yes\" -->"));
        assert!(graph.contains("-- \"No\" -->"));
    }

    #[test]
    fn test_generate_cfg_all_variants_coverage() {
        use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType};

        // Construct dummy AnalyzedExpr
        let dummy_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        };

        // Construct a program with all remaining statement variants manually
        let program = AnalyzedProgram {
            statements: vec![
                AnalyzedStatement::Assignment { name: "x".into(), value: dummy_expr.clone() },
                AnalyzedStatement::Expression(vec![dummy_expr.clone()]),
                AnalyzedStatement::Query(vec![dummy_expr.clone()]),
                AnalyzedStatement::For {
                    variable: "i".into(),
                    iterator: Box::new(dummy_expr.clone()),
                    body: vec![],
                },
                AnalyzedStatement::Match {
                    scrutinee: Box::new(dummy_expr.clone()),
                    arms: vec![(dummy_expr.clone(), vec![])],
                },
                AnalyzedStatement::Break,
                AnalyzedStatement::Continue,
                AnalyzedStatement::Return { value: Some(Box::new(dummy_expr.clone())) },
                AnalyzedStatement::FunctionDef {
                    name: "func".into(),
                    params: vec![],
                    body: vec![],
                    return_type: None,
                },
                AnalyzedStatement::TypeDefinition {
                    name: "Type".into(),
                    fields: vec![],
                },
                AnalyzedStatement::TraitDefinition {
                    name: "Trait".into(),
                    methods: vec![],
                },
                AnalyzedStatement::TraitImplementation {
                    trait_name: "Trait".into(),
                    type_name: "Type".into(),
                    methods: vec![],
                },
                AnalyzedStatement::TestDeclaration {
                    name: "test".into(),
                    body: vec![],
                },
            ],
            scope: crate::semantic::Scope::new(),
        };

        let graph = generate_cfg(&program);

        assert!(graph.contains("Assign x = ..."));
        assert!(graph.contains("Expression"));
        assert!(graph.contains("Query"));
        assert!(graph.contains("For i in ..."));
        assert!(graph.contains("Match"));
        assert!(graph.contains("Arm 0"));
        assert!(graph.contains("Break"));
        assert!(graph.contains("Continue"));
        assert!(graph.contains("Return"));
        assert!(graph.contains("Fn func"));
        assert!(graph.contains("Type Type"));
        assert!(graph.contains("Trait Trait"));
        assert!(graph.contains("Impl Trait for Type"));
        assert!(graph.contains("Test test"));
    }
}
