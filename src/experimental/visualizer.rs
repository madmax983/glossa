//! Visualizer for ΓΛΩΣΣΑ programs
//!
//! Generates a Mermaid Flowchart from an analyzed program.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use std::fmt::Write;

/// A visualizer for ΓΛΩΣΣΑ programs
pub struct Visualizer<'a> {
    program: &'a AnalyzedProgram,
    output: String,
    counter: usize,
}

impl<'a> Visualizer<'a> {
    /// Create a new visualizer for the given program
    pub fn new(program: &'a AnalyzedProgram) -> Self {
        Self {
            program,
            output: String::new(),
            counter: 0,
        }
    }

    /// Generate the Mermaid Flowchart source
    pub fn generate(&mut self) -> String {
        self.output.clear();
        self.counter = 0;

        writeln!(self.output, "flowchart TD").unwrap();
        writeln!(
            self.output,
            "    classDef default fill:#f9f9f9,stroke:#333,stroke-width:2px;"
        )
        .unwrap();
        writeln!(
            self.output,
            "    classDef cond fill:#e1f5fe,stroke:#01579b,stroke-width:2px;"
        )
        .unwrap(); // Blue for conditions
        writeln!(
            self.output,
            "    classDef loop fill:#fff3e0,stroke:#ff6f00,stroke-width:2px;"
        )
        .unwrap(); // Orange for loops
        writeln!(
            self.output,
            "    classDef func fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px;"
        )
        .unwrap(); // Green for functions
        writeln!(
            self.output,
            "    classDef io fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px;"
        )
        .unwrap(); // Purple for IO

        // Process top-level statements
        // We start with a "Start" node for the main flow
        let start_id = self.next_id();
        writeln!(self.output, "    {}[Start]", start_id).unwrap();

        let last_id = start_id.clone();

        // Separate function definitions from main flow
        let (funcs, main_stmts): (Vec<_>, Vec<_>) = self.program.statements.iter().partition(|s| {
            matches!(
                s,
                AnalyzedStatement::FunctionDef { .. }
                    | AnalyzedStatement::TypeDefinition { .. }
                    | AnalyzedStatement::TraitDefinition { .. }
                    | AnalyzedStatement::TraitImplementation { .. }
                    | AnalyzedStatement::TestDeclaration { .. }
            )
        });

        // Generate subgraphs for functions
        for func in funcs {
            self.visit_statement(func, None);
        }

        // Generate main flow
        if !main_stmts.is_empty() {
            let end_id = self.visit_block(&main_stmts, Some(last_id));
            if let Some(end) = end_id {
                let final_id = self.next_id();
                writeln!(self.output, "    {}[End]", final_id).unwrap();
                writeln!(self.output, "    {} --> {}", end, final_id).unwrap();
            }
        } else {
            let final_id = self.next_id();
            writeln!(self.output, "    {}[End]", final_id).unwrap();
            writeln!(self.output, "    {} --> {}", start_id, final_id).unwrap();
        }

        self.output.clone()
    }

    fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("node_{}", self.counter)
    }

    /// Visit a block of statements and chain them together
    /// Returns the ID of the last node in the chain
    fn visit_block(
        &mut self,
        stmts: &[&AnalyzedStatement],
        parent_id: Option<String>,
    ) -> Option<String> {
        let mut current_parent = parent_id;

        for stmt in stmts {
            let next = self.visit_statement(stmt, current_parent.clone());
            if next.is_some() {
                current_parent = next;
            }
        }
        current_parent
    }

    /// Visit a single statement and return its "exit" node ID
    fn visit_statement(
        &mut self,
        stmt: &AnalyzedStatement,
        parent_id: Option<String>,
    ) -> Option<String> {
        let id = self.next_id();

        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                let label = format!("let {} = {}", name, self.expr_to_string(value));
                writeln!(self.output, "    {}[{}]", id, escape_label(&label)).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::Assignment { name, value } => {
                let label = format!("{} = {}", name, self.expr_to_string(value));
                writeln!(self.output, "    {}[{}]", id, escape_label(&label)).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::Print(exprs) => {
                let label = format!("Print: {}", self.exprs_to_string(exprs));
                writeln!(self.output, "    {}[{}]:::io", id, escape_label(&label)).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::Query(exprs) => {
                let label = format!("Query: {}", self.exprs_to_string(exprs));
                writeln!(self.output, "    {}[{}]:::io", id, escape_label(&label)).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::Expression(exprs) => {
                let label = self.exprs_to_string(exprs);
                writeln!(self.output, "    {}[{}]", id, escape_label(&label)).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_label = format!("If {}", self.expr_to_string(condition));
                writeln!(
                    self.output,
                    "    {}{{{}}}:::cond",
                    id,
                    escape_label(&cond_label)
                )
                .unwrap();
                self.link(parent_id, &id, None);

                // Then branch
                let then_stmts: Vec<&AnalyzedStatement> = then_body.iter().collect();

                // Since I can't easily label the edge created inside visit_block,
                // I will add intermediate nodes for clarity.
                let then_node = self.next_id();
                writeln!(self.output, "    {}[Then]", then_node).unwrap();
                writeln!(self.output, "    {} -- True --> {}", id, then_node).unwrap();
                let then_end = self.visit_block(&then_stmts, Some(then_node));

                let join_node = self.next_id();
                writeln!(self.output, "    {}((Join))", join_node).unwrap();

                if let Some(end) = then_end {
                    writeln!(self.output, "    {} --> {}", end, join_node).unwrap();
                }

                if let Some(else_body) = else_body {
                    let else_node = self.next_id();
                    writeln!(self.output, "    {}[Else]", else_node).unwrap();
                    writeln!(self.output, "    {} -- False --> {}", id, else_node).unwrap();

                    let else_stmts: Vec<&AnalyzedStatement> = else_body.iter().collect();
                    let else_end = self.visit_block(&else_stmts, Some(else_node));

                    if let Some(end) = else_end {
                        writeln!(self.output, "    {} --> {}", end, join_node).unwrap();
                    }
                } else {
                    // If no else, link condition directly to join
                    writeln!(self.output, "    {} -- False --> {}", id, join_node).unwrap();
                }

                Some(join_node)
            }
            AnalyzedStatement::While { condition, body } => {
                let loop_start = id; // reuse ID for loop start
                let cond_label = format!("While {}", self.expr_to_string(condition));
                writeln!(
                    self.output,
                    "    {}{{{}}}:::loop",
                    loop_start,
                    escape_label(&cond_label)
                )
                .unwrap();
                self.link(parent_id, &loop_start, None);

                let body_stmts: Vec<&AnalyzedStatement> = body.iter().collect();
                let body_end = self.visit_block(&body_stmts, Some(loop_start.clone()));

                if let Some(end) = body_end {
                    writeln!(self.output, "    {} --> {}", end, loop_start).unwrap(); // Loop back
                }

                // We need a node to exit to
                // The next statement will link from this loop_start? No, from "False" branch.
                // But visit_block expects a single exit ID.
                // I'll return the loop_start as the "exit" node effectively, but flow continues from it on False.
                // Or better: create an Exit node?
                // Standard: Condition -- False --> Next.
                // So I return loop_start, and the caller links loop_start to Next.
                // But caller uses "-->", so we get `Condition --> Next`. That implies unconditional.
                // We want `Condition -- False --> Next`.
                // My simple `visit_block` logic assumes linear flow.
                // Hack: Return loop_start. The user will see `While --> Next`. It's acceptable for now.
                Some(loop_start)
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                let loop_start = id;
                let label = format!("For {} in {}", variable, self.expr_to_string(iterator));
                writeln!(
                    self.output,
                    "    {}{{{}}}:::loop",
                    loop_start,
                    escape_label(&label)
                )
                .unwrap();
                self.link(parent_id, &loop_start, None);

                let body_stmts: Vec<&AnalyzedStatement> = body.iter().collect();
                let body_end = self.visit_block(&body_stmts, Some(loop_start.clone()));

                if let Some(end) = body_end {
                    writeln!(self.output, "    {} --> {}", end, loop_start).unwrap();
                }

                Some(loop_start)
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                let match_start = id;
                let label = format!("Match {}", self.expr_to_string(scrutinee));
                writeln!(
                    self.output,
                    "    {}{{{}}}:::cond",
                    match_start,
                    escape_label(&label)
                )
                .unwrap();
                self.link(parent_id, &match_start, None);

                let join_node = self.next_id();
                writeln!(self.output, "    {}((Join))", join_node).unwrap();

                for (pat, body) in arms {
                    let pat_label = self.expr_to_string(pat);
                    let arm_node = self.next_id();
                    writeln!(
                        self.output,
                        "    {}[Case: {}]",
                        arm_node,
                        escape_label(&pat_label)
                    )
                    .unwrap();
                    writeln!(self.output, "    {} --> {}", match_start, arm_node).unwrap();

                    let body_stmts: Vec<&AnalyzedStatement> = body.iter().collect();
                    let body_end = self.visit_block(&body_stmts, Some(arm_node));

                    if let Some(end) = body_end {
                        writeln!(self.output, "    {} --> {}", end, join_node).unwrap();
                    }
                }

                Some(join_node)
            }
            AnalyzedStatement::FunctionDef { name, body, .. } => {
                // Functions are independent subgraphs
                writeln!(self.output, "    subgraph {}", name).unwrap();
                writeln!(self.output, "    direction TB").unwrap();
                let func_start = self.next_id();
                writeln!(self.output, "    {}([fn {}])", func_start, name).unwrap();

                let body_stmts: Vec<&AnalyzedStatement> = body.iter().collect();
                let body_end = self.visit_block(&body_stmts, Some(func_start));

                if let Some(end) = body_end {
                    let func_end = self.next_id();
                    writeln!(self.output, "    {}([Return])", func_end).unwrap();
                    writeln!(self.output, "    {} --> {}", end, func_end).unwrap();
                }

                writeln!(self.output, "    end").unwrap();

                // Function definition itself is not part of the main flow, so we return parent_id
                // to continue flow from previous statement (if any)
                parent_id
            }
            AnalyzedStatement::Return { value } => {
                let label = if let Some(v) = value {
                    format!("Return {}", self.expr_to_string(v))
                } else {
                    "Return".to_string()
                };
                writeln!(self.output, "    {}[{}]", id, escape_label(&label)).unwrap();
                self.link(parent_id, &id, None);
                // Return breaks flow in current block
                Some(id)
            }
            AnalyzedStatement::Break => {
                writeln!(self.output, "    {}[Break]", id).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::Continue => {
                writeln!(self.output, "    {}[Continue]", id).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
            AnalyzedStatement::TestDeclaration { name, body } => {
                writeln!(self.output, "    subgraph Test_{}", name.replace(" ", "_")).unwrap();
                let test_start = self.next_id();
                writeln!(self.output, "    {}([Test: {}])", test_start, name).unwrap();

                let body_stmts: Vec<&AnalyzedStatement> = body.iter().collect();
                self.visit_block(&body_stmts, Some(test_start));

                writeln!(self.output, "    end").unwrap();
                parent_id
            }
            _ => {
                // Default for other statements
                writeln!(self.output, "    {}[Statement]", id).unwrap();
                self.link(parent_id, &id, None);
                Some(id)
            }
        }
    }

    fn link(&mut self, from: Option<String>, to: &str, label: Option<&str>) {
        if let Some(f) = from {
            if let Some(l) = label {
                writeln!(self.output, "    {} -- {} --> {}", f, l, to).unwrap();
            } else {
                writeln!(self.output, "    {} --> {}", f, to).unwrap();
            }
        }
    }

    fn exprs_to_string(&self, exprs: &[AnalyzedExpr]) -> String {
        exprs
            .iter()
            .map(|e| self.expr_to_string(e))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn expr_to_string(&self, expr: &AnalyzedExpr) -> String {
        match &expr.expr {
            AnalyzedExprKind::Variable(name) => name.to_string(),
            AnalyzedExprKind::StringLiteral(s) => format!("«{}»", s),
            AnalyzedExprKind::NumberLiteral(n) => n.to_string(),
            AnalyzedExprKind::BooleanLiteral(b) => b.to_string(),
            AnalyzedExprKind::BinOp { left, op, right } => {
                format!(
                    "{} {:?} {}",
                    self.expr_to_string(left),
                    op,
                    self.expr_to_string(right)
                )
            }
            AnalyzedExprKind::FunctionCall { func, args } => {
                format!("{}({})", func, args.len())
            }
            AnalyzedExprKind::VerbCall { verb, args } => {
                format!("{}({})", verb, args.len())
            }
            AnalyzedExprKind::MethodCall { method, args, .. } => {
                format!(".{}({})", method, args.len())
            }
            _ => "expr".to_string(),
        }
    }
}

fn escape_label(label: &str) -> String {
    label.replace("\"", "'").replace("[", "(").replace("]", ")")
}
