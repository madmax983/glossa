//! The Labyrinth Tool ("Labyrinth")
//!
//! This module implements the "Labyrinth" functionality, which visualizes the
//! control flow graph of a ΓΛΩΣΣΑ program as a Mermaid.js flowchart.
//!
//! # Purpose
//!
//! "Architectural Transparency" is a core value of the project.
//! As programs grow, the control flow (loops, ifs, function calls)
//! can become complex. The Labyrinth renders these paths visible.
//!
//! # The Map
//!
//! The output is a standard Mermaid Flowchart Diagram that shows:
//! * **Nodes**: Statements and expressions.
//! * **Edges**: The flow of execution.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, analyze_program};
use crate::tools::ui::Status;
use std::path::Path;

/// Run the Labyrinth tool on a file
pub fn run_labyrinth(input: &Path) -> miette::Result<()> {
    let source = crate::tools::runner::load_source(input)?;

    let status = Status::start_with_symbol("Λαβύρινθος (Control Flow Graph)", "🔀");

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

    let cfg = generate_cfg(&program);
    status.success();

    println!("\n```mermaid");
    println!("{}", cfg);
    println!("```\n");

    Ok(())
}

pub fn generate_cfg(program: &AnalyzedProgram) -> String {
    let mut builder = CFGBuilder::new();
    builder.build_program(program);
    builder.finish()
}

struct CFGBuilder {
    nodes: Vec<String>,
    edges: Vec<String>,
    node_counter: usize,
}

impl CFGBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_counter: 0,
        }
    }

    fn next_node_id(&mut self) -> String {
        self.node_counter += 1;
        format!("N{}", self.node_counter)
    }

    fn add_node(&mut self, label: &str, shape: &str) -> String {
        let id = self.next_node_id();
        let formatted = match shape {
            "diamond" => format!("{}{{\"{}\"}}", id, label),
            "rect" => format!("{}[\"{}\"]", id, label),
            "round" => format!("{}(\"{}\")", id, label),
            _ => format!("{}[\"{}\"]", id, label),
        };
        self.nodes.push(formatted);
        id
    }

    fn add_edge(&mut self, from: &str, to: &str, label: Option<&str>) {
        if let Some(l) = label {
            self.edges.push(format!("{} -- {} --> {}", from, l, to));
        } else {
            self.edges.push(format!("{} --> {}", from, to));
        }
    }

    fn build_program(&mut self, program: &AnalyzedProgram) {
        let start_node = self.add_node("Start", "round");
        let mut current_node = start_node;

        for stmt in &program.statements {
            current_node = self.build_statement(stmt, &current_node);
        }

        let end_node = self.add_node("End", "round");
        self.add_edge(&current_node, &end_node, None);
    }

    fn build_statement(&mut self, stmt: &AnalyzedStatement, prev_node: &str) -> String {
        match stmt {
            AnalyzedStatement::Binding { name, .. } => {
                let id = self.add_node(&format!("Binding: {}", name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::Assignment { name, .. } => {
                let id = self.add_node(&format!("Assignment: {}", name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::Print(_) => {
                let id = self.add_node("Print", "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::Expression(_) => {
                let id = self.add_node("Expression", "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::Query(_) => {
                let id = self.add_node("Query", "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::If {
                then_body,
                else_body,
                ..
            } => {
                let cond_id = self.add_node("If Condition", "diamond");
                self.add_edge(prev_node, &cond_id, None);

                let end_id = self.add_node("End If", "round");

                // Then branch
                let mut current_then = cond_id.clone();
                let mut then_has_nodes = false;
                for sub_stmt in then_body {
                    let next_node = self.build_statement(sub_stmt, &current_then);
                    if !then_has_nodes {
                        // Fix the edge from condition to first statement of then branch
                        // by removing the previously added unconditional edge
                        self.edges.pop();
                        self.add_edge(&cond_id, &next_node, Some("Yes"));
                    }
                    current_then = next_node;
                    then_has_nodes = true;
                }

                if !then_has_nodes {
                    self.add_edge(&cond_id, &end_id, Some("Yes"));
                } else {
                    self.add_edge(&current_then, &end_id, None);
                }

                // Else branch
                if let Some(else_b) = else_body {
                    let mut current_else = cond_id.clone();
                    let mut else_has_nodes = false;
                    for sub_stmt in else_b {
                        let next_node = self.build_statement(sub_stmt, &current_else);
                        if !else_has_nodes {
                            self.edges.pop();
                            self.add_edge(&cond_id, &next_node, Some("No"));
                        }
                        current_else = next_node;
                        else_has_nodes = true;
                    }
                    if !else_has_nodes {
                        self.add_edge(&cond_id, &end_id, Some("No"));
                    } else {
                        self.add_edge(&current_else, &end_id, None);
                    }
                } else {
                    self.add_edge(&cond_id, &end_id, Some("No"));
                }

                end_id
            }
            AnalyzedStatement::While { body, .. } => {
                let cond_id = self.add_node("While Condition", "diamond");
                self.add_edge(prev_node, &cond_id, None);

                let end_id = self.add_node("End While", "round");

                let mut current_body = cond_id.clone();
                let mut has_nodes = false;
                for sub_stmt in body {
                    let next_node = self.build_statement(sub_stmt, &current_body);
                    if !has_nodes {
                        self.edges.pop();
                        self.add_edge(&cond_id, &next_node, Some("Yes"));
                    }
                    current_body = next_node;
                    has_nodes = true;
                }

                if has_nodes {
                    self.add_edge(&current_body, &cond_id, None);
                } else {
                    self.add_edge(&cond_id, &cond_id, Some("Yes"));
                }
                self.add_edge(&cond_id, &end_id, Some("No"));

                end_id
            }
            AnalyzedStatement::For { variable, body, .. } => {
                let cond_id = self.add_node(&format!("For: {}", variable), "diamond");
                self.add_edge(prev_node, &cond_id, None);

                let end_id = self.add_node("End For", "round");

                let mut current_body = cond_id.clone();
                let mut has_nodes = false;
                for sub_stmt in body {
                    let next_node = self.build_statement(sub_stmt, &current_body);
                    if !has_nodes {
                        self.edges.pop();
                        self.add_edge(&cond_id, &next_node, Some("Next"));
                    }
                    current_body = next_node;
                    has_nodes = true;
                }

                if has_nodes {
                    self.add_edge(&current_body, &cond_id, None);
                } else {
                    self.add_edge(&cond_id, &cond_id, Some("Next"));
                }
                self.add_edge(&cond_id, &end_id, Some("Done"));

                end_id
            }
            AnalyzedStatement::Match { arms, .. } => {
                let match_id = self.add_node("Match", "diamond");
                self.add_edge(prev_node, &match_id, None);
                let end_id = self.add_node("End Match", "round");

                for (i, (_, body)) in arms.iter().enumerate() {
                    let mut current_arm = match_id.clone();
                    let mut has_nodes = false;
                    for sub_stmt in body {
                        let next_node = self.build_statement(sub_stmt, &current_arm);
                        if !has_nodes {
                            self.edges.pop();
                            self.add_edge(&match_id, &next_node, Some(&format!("Arm {}", i)));
                        }
                        current_arm = next_node;
                        has_nodes = true;
                    }
                    if has_nodes {
                        self.add_edge(&current_arm, &end_id, None);
                    } else {
                        self.add_edge(&match_id, &end_id, Some(&format!("Arm {}", i)));
                    }
                }

                end_id
            }
            AnalyzedStatement::Break => {
                let id = self.add_node("Break", "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::Continue => {
                let id = self.add_node("Continue", "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::Return { .. } => {
                let id = self.add_node("Return", "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::FunctionDef { name, .. } => {
                let id = self.add_node(&format!("Function: {}", name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::TypeDefinition { name, .. } => {
                let id = self.add_node(&format!("Type: {}", name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::TraitDefinition { name, .. } => {
                let id = self.add_node(&format!("Trait: {}", name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::TraitImplementation {
                type_name,
                trait_name,
                ..
            } => {
                let id = self.add_node(&format!("Impl {} for {}", trait_name, type_name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
            AnalyzedStatement::TestDeclaration { name, .. } => {
                let id = self.add_node(&format!("Test: {}", name), "rect");
                self.add_edge(prev_node, &id, None);
                id
            }
        }
    }

    fn finish(self) -> String {
        let mut out = String::from("graph TD\n");
        for node in self.nodes {
            out.push_str(&format!("    {}\n", node));
        }
        for edge in self.edges {
            out.push_str(&format!("    {}\n", edge));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, GlossaType, Scope};
    use smol_str::SmolStr;

    fn dummy_expr() -> AnalyzedExpr {
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        }
    }

    #[test]
    #[allow(clippy::vec_init_then_push)]
    fn test_generate_cfg_all_arms() {
        let mut statements = vec![];

        statements.push(AnalyzedStatement::Binding {
            name: SmolStr::new("x"),
            value: dummy_expr(),
            mutable: false,
        });

        statements.push(AnalyzedStatement::Assignment {
            name: SmolStr::new("x"),
            value: dummy_expr(),
        });

        statements.push(AnalyzedStatement::Print(vec![dummy_expr()]));
        statements.push(AnalyzedStatement::Expression(vec![dummy_expr()]));
        statements.push(AnalyzedStatement::Query(vec![dummy_expr()]));

        statements.push(AnalyzedStatement::If {
            condition: Box::new(dummy_expr()),
            then_body: vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
            else_body: Some(vec![AnalyzedStatement::Expression(vec![dummy_expr()])]),
        });

        statements.push(AnalyzedStatement::If {
            condition: Box::new(dummy_expr()),
            then_body: vec![],
            else_body: None,
        });

        statements.push(AnalyzedStatement::While {
            condition: Box::new(dummy_expr()),
            body: vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
        });

        statements.push(AnalyzedStatement::While {
            condition: Box::new(dummy_expr()),
            body: vec![],
        });

        statements.push(AnalyzedStatement::For {
            variable: SmolStr::new("i"),
            iterator: Box::new(dummy_expr()),
            body: vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
        });

        statements.push(AnalyzedStatement::For {
            variable: SmolStr::new("i"),
            iterator: Box::new(dummy_expr()),
            body: vec![],
        });

        statements.push(AnalyzedStatement::Match {
            scrutinee: Box::new(dummy_expr()),
            arms: vec![
                (
                    dummy_expr(),
                    vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
                ),
                (dummy_expr(), vec![]),
            ],
        });

        statements.push(AnalyzedStatement::Break);
        statements.push(AnalyzedStatement::Continue);
        statements.push(AnalyzedStatement::Return { value: None });

        statements.push(AnalyzedStatement::FunctionDef {
            name: SmolStr::new("my_fn"),
            params: vec![],
            body: vec![],
            return_type: None,
        });

        statements.push(AnalyzedStatement::TypeDefinition {
            name: SmolStr::new("MyType"),
            fields: vec![],
        });

        statements.push(AnalyzedStatement::TraitDefinition {
            name: SmolStr::new("MyTrait"),
            methods: vec![],
        });

        statements.push(AnalyzedStatement::TraitImplementation {
            type_name: SmolStr::new("MyType"),
            trait_name: SmolStr::new("MyTrait"),
            methods: vec![],
        });

        statements.push(AnalyzedStatement::TestDeclaration {
            name: "my_test".to_string(),
            body: vec![],
        });

        let program = AnalyzedProgram {
            statements,
            scope: Scope::new(),
        };

        let cfg = generate_cfg(&program);

        assert!(cfg.contains("graph TD"));
        assert!(cfg.contains("Start"));
        assert!(cfg.contains("End"));
        assert!(cfg.contains("Binding: x"));
        assert!(cfg.contains("Assignment: x"));
        assert!(cfg.contains("Print"));
        assert!(cfg.contains("Expression"));
        assert!(cfg.contains("Query"));
        assert!(cfg.contains("If Condition"));
        assert!(cfg.contains("Yes"));
        assert!(cfg.contains("No"));
        assert!(cfg.contains("While Condition"));
        assert!(cfg.contains("For: i"));
        assert!(cfg.contains("Next"));
        assert!(cfg.contains("Done"));
        assert!(cfg.contains("Match"));
        assert!(cfg.contains("Arm 0"));
        assert!(cfg.contains("Arm 1"));
        assert!(cfg.contains("Break"));
        assert!(cfg.contains("Continue"));
        assert!(cfg.contains("Return"));
        assert!(cfg.contains("Function: my_fn"));
        assert!(cfg.contains("Type: MyType"));
        assert!(cfg.contains("Trait: MyTrait"));
        assert!(cfg.contains("Impl MyTrait for MyType"));
        assert!(cfg.contains("Test: my_test"));
    }
}

#[test]
fn test_run_labyrinth_errors() {
    use std::io::Write;
    // Test parsing error
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    write!(temp_file, "invalid syntax").unwrap();
    let result = run_labyrinth(temp_file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Parse"));

    // Test semantic error (valid parse, invalid semantics like undef var)
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    write!(temp_file, "ψ 10 γίγνεται.").unwrap(); // Reassigning undef var
    let result = run_labyrinth(temp_file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("ὡρίσθη"));
}
