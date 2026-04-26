//! The Haruspex (ὁ Ἱεροσκόπος) - Graphviz AST Visualizer
//!
//! This module implements the "Haruspex" tool, which inspects the semantic AST
//! (`AnalyzedProgram`) of a ΓΛΩΣΣΑ program and translates it into a DOT graph
//! for visualization with Graphviz.
//!
//! # Purpose
//!
//! While the Cartographer maps architecture and the Labyrinth traces control flow,
//! the Haruspex allows compiler developers to inspect the raw semantic tree
//! structure, seeing exactly how expressions are nested and typed.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Haruspex tool to generate a Graphviz DOT representation of the AST.
pub fn run_haruspex(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἱεροσκόπος (Generating DOT Graph)", "👁️");

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

    let mut generator = DotGenerator::new();
    let dot = generator.generate(&program);

    println!("{}", dot);
    Ok(())
}

struct DotGenerator {
    next_id: usize,
    output: String,
}

impl DotGenerator {
    fn new() -> Self {
        Self {
            next_id: 0,
            output: String::new(),
        }
    }

    fn generate(&mut self, program: &AnalyzedProgram) -> String {
        self.output.push_str("digraph AST {\n");
        self.output.push_str(
            "    node [shape=box, style=filled, fillcolor=lightgrey, fontname=\"Courier\"];\n",
        );
        self.output.push_str("    edge [fontname=\"Courier\"];\n");

        let root_id = self.next_id();
        self.emit_node(root_id, "Program", "lightblue");

        for stmt in &program.statements {
            let stmt_id = self.visit_statement(stmt);
            self.emit_edge(root_id, stmt_id, "");
        }

        self.output.push_str("}\n");
        self.output.clone()
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn emit_node(&mut self, id: usize, label: &str, color: &str) {
        // Escape quotes
        let safe_label = label.replace("\"", "\\\"");
        let _ = writeln!(
            &mut self.output,
            "    node_{} [label=\"{}\", fillcolor=\"{}\"];",
            id, safe_label, color
        );
    }

    fn emit_edge(&mut self, from: usize, to: usize, label: &str) {
        if label.is_empty() {
            let _ = writeln!(&mut self.output, "    node_{} -> node_{};", from, to);
        } else {
            let safe_label = label.replace("\"", "\\\"");
            let _ = writeln!(
                &mut self.output,
                "    node_{} -> node_{} [label=\"{}\"];",
                from, to, safe_label
            );
        }
    }

    fn visit_statement(&mut self, stmt: &AnalyzedStatement) -> usize {
        let id = self.next_id();

        match stmt {
            AnalyzedStatement::Binding {
                name,
                value,
                mutable,
            } => {
                let mut_str = if *mutable { "mut " } else { "" };
                self.emit_node(id, &format!("Binding\\n{}{}", mut_str, name), "lightgreen");
                let val_id = self.visit_expr(value);
                self.emit_edge(id, val_id, "value");
            }
            AnalyzedStatement::Assignment { name, value } => {
                self.emit_node(id, &format!("Assignment\\n{}", name), "lightgreen");
                let val_id = self.visit_expr(value);
                self.emit_edge(id, val_id, "value");
            }
            AnalyzedStatement::Print(exprs) => {
                self.emit_node(id, "Print", "lightgreen");
                for (i, expr) in exprs.iter().enumerate() {
                    let child_id = self.visit_expr(expr);
                    self.emit_edge(id, child_id, &format!("arg_{}", i));
                }
            }
            AnalyzedStatement::Expression(exprs) => {
                self.emit_node(id, "ExpressionStmt", "lightgreen");
                for (i, expr) in exprs.iter().enumerate() {
                    let child_id = self.visit_expr(expr);
                    self.emit_edge(id, child_id, &format!("expr_{}", i));
                }
            }
            AnalyzedStatement::Query(exprs) => {
                self.emit_node(id, "Query", "lightgreen");
                for (i, expr) in exprs.iter().enumerate() {
                    let child_id = self.visit_expr(expr);
                    self.emit_edge(id, child_id, &format!("arg_{}", i));
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.emit_node(id, "If", "lightgreen");
                let cond_id = self.visit_expr(condition);
                self.emit_edge(id, cond_id, "condition");

                let then_id = self.next_id();
                self.emit_node(then_id, "Block (Then)", "white");
                self.emit_edge(id, then_id, "then");
                for s in then_body {
                    let s_id = self.visit_statement(s);
                    self.emit_edge(then_id, s_id, "");
                }

                if let Some(else_b) = else_body {
                    let else_id = self.next_id();
                    self.emit_node(else_id, "Block (Else)", "white");
                    self.emit_edge(id, else_id, "else");
                    for s in else_b {
                        let s_id = self.visit_statement(s);
                        self.emit_edge(else_id, s_id, "");
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                self.emit_node(id, "While", "lightgreen");
                let cond_id = self.visit_expr(condition);
                self.emit_edge(id, cond_id, "condition");

                let body_id = self.next_id();
                self.emit_node(body_id, "Block", "white");
                self.emit_edge(id, body_id, "body");
                for s in body {
                    let s_id = self.visit_statement(s);
                    self.emit_edge(body_id, s_id, "");
                }
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                self.emit_node(id, &format!("For\\n{}", variable), "lightgreen");
                let iter_id = self.visit_expr(iterator);
                self.emit_edge(id, iter_id, "iterator");

                let body_id = self.next_id();
                self.emit_node(body_id, "Block", "white");
                self.emit_edge(id, body_id, "body");
                for s in body {
                    let s_id = self.visit_statement(s);
                    self.emit_edge(body_id, s_id, "");
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.emit_node(id, "Match", "lightgreen");
                let scrutinee_id = self.visit_expr(scrutinee);
                self.emit_edge(id, scrutinee_id, "scrutinee");

                for (i, (pat, body)) in arms.iter().enumerate() {
                    let arm_id = self.next_id();
                    self.emit_node(arm_id, &format!("Arm_{}", i), "white");
                    self.emit_edge(id, arm_id, "");

                    let pat_id = self.visit_expr(pat);
                    self.emit_edge(arm_id, pat_id, "pattern");

                    for s in body {
                        let s_id = self.visit_statement(s);
                        self.emit_edge(arm_id, s_id, "body");
                    }
                }
            }
            AnalyzedStatement::Break => self.emit_node(id, "Break", "lightgreen"),
            AnalyzedStatement::Continue => self.emit_node(id, "Continue", "lightgreen"),
            AnalyzedStatement::Return { value } => {
                self.emit_node(id, "Return", "lightgreen");
                if let Some(val) = value {
                    let val_id = self.visit_expr(val);
                    self.emit_edge(id, val_id, "value");
                }
            }
            AnalyzedStatement::FunctionDef {
                name,
                params,
                body,
                return_type,
            } => {
                let ret_str = return_type
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "()".to_string());
                self.emit_node(
                    id,
                    &format!("FunctionDef\\n{} -> {}", name, ret_str),
                    "lightgreen",
                );

                for (i, (p_name, p_type)) in params.iter().enumerate() {
                    let type_str = p_type
                        .as_ref()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "?".to_string());
                    let p_id = self.next_id();
                    self.emit_node(p_id, &format!("{}: {}", p_name, type_str), "lightyellow");
                    self.emit_edge(id, p_id, &format!("param_{}", i));
                }

                let body_id = self.next_id();
                self.emit_node(body_id, "Block", "white");
                self.emit_edge(id, body_id, "body");
                for s in body {
                    let s_id = self.visit_statement(s);
                    self.emit_edge(body_id, s_id, "");
                }
            }
            AnalyzedStatement::TypeDefinition { name, fields } => {
                self.emit_node(id, &format!("TypeDefinition\\n{}", name), "lightgreen");
                for (i, (f_name, f_type)) in fields.iter().enumerate() {
                    let f_id = self.next_id();
                    self.emit_node(f_id, &format!("{}: {}", f_name, f_type), "lightyellow");
                    self.emit_edge(id, f_id, &format!("field_{}", i));
                }
            }
            AnalyzedStatement::TraitDefinition { name, methods } => {
                self.emit_node(id, &format!("TraitDefinition\\n{}", name), "lightgreen");
                for (i, method) in methods.iter().enumerate() {
                    let m_id = self.next_id();
                    self.emit_node(
                        m_id,
                        &format!("MethodDecl\\n{}", method.name),
                        "lightyellow",
                    );
                    self.emit_edge(id, m_id, &format!("method_{}", i));
                }
            }
            AnalyzedStatement::TraitImplementation {
                trait_name,
                type_name,
                methods,
            } => {
                self.emit_node(
                    id,
                    &format!("TraitImpl\\n{} for {}", trait_name, type_name),
                    "lightgreen",
                );
                for (i, method) in methods.iter().enumerate() {
                    let m_id = self.next_id();
                    self.emit_node(
                        m_id,
                        &format!("MethodImpl\\n{}", method.name),
                        "lightyellow",
                    );
                    self.emit_edge(id, m_id, &format!("method_{}", i));
                }
            }
            AnalyzedStatement::TestDeclaration { name, body } => {
                self.emit_node(id, &format!("TestDeclaration\\n{}", name), "lightgreen");
                let body_id = self.next_id();
                self.emit_node(body_id, "Block", "white");
                self.emit_edge(id, body_id, "body");
                for s in body {
                    let s_id = self.visit_statement(s);
                    self.emit_edge(body_id, s_id, "");
                }
            }
        }

        id
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr) -> usize {
        let id = self.next_id();

        // Include type information in expression nodes
        let type_info = format!("\\n[{}]", expr.glossa_type);

        match &expr.expr {
            AnalyzedExprKind::StringLiteral(s) => {
                self.emit_node(
                    id,
                    &format!("String\\n\\\"{}\\\"{}", s, type_info),
                    "lightyellow",
                );
            }
            AnalyzedExprKind::NumberLiteral(n) => {
                self.emit_node(id, &format!("Number\\n{}{}", n, type_info), "lightyellow");
            }
            AnalyzedExprKind::BooleanLiteral(b) => {
                self.emit_node(id, &format!("Boolean\\n{}{}", b, type_info), "lightyellow");
            }
            AnalyzedExprKind::Variable(v) => {
                self.emit_node(id, &format!("Variable\\n{}{}", v, type_info), "lightyellow");
            }
            AnalyzedExprKind::PropertyAccess { owner, property } => {
                self.emit_node(
                    id,
                    &format!("PropertyAccess\\n.{}{}", property, type_info),
                    "lightyellow",
                );
                let owner_id = self.visit_expr(owner);
                self.emit_edge(id, owner_id, "owner");
            }
            AnalyzedExprKind::VerbCall { verb, args } => {
                self.emit_node(
                    id,
                    &format!("VerbCall\\n{}{}", verb, type_info),
                    "lightyellow",
                );
                for (i, arg) in args.iter().enumerate() {
                    let arg_id = self.visit_expr(arg);
                    self.emit_edge(id, arg_id, &format!("arg_{}", i));
                }
            }
            AnalyzedExprKind::BinOp { left, op, right } => {
                self.emit_node(id, &format!("BinOp\\n{:?}{}", op, type_info), "lightyellow");
                let left_id = self.visit_expr(left);
                let right_id = self.visit_expr(right);
                self.emit_edge(id, left_id, "left");
                self.emit_edge(id, right_id, "right");
            }
            AnalyzedExprKind::UnaryOp { op, operand } => {
                self.emit_node(
                    id,
                    &format!("UnaryOp\\n{:?}{}", op, type_info),
                    "lightyellow",
                );
                let operand_id = self.visit_expr(operand);
                self.emit_edge(id, operand_id, "operand");
            }
            AnalyzedExprKind::Range {
                start,
                end,
                inclusive,
            } => {
                let range_sym = if *inclusive { "..=" } else { ".." };
                self.emit_node(
                    id,
                    &format!("Range\\n{}{}", range_sym, type_info),
                    "lightyellow",
                );
                let start_id = self.visit_expr(start);
                let end_id = self.visit_expr(end);
                self.emit_edge(id, start_id, "start");
                self.emit_edge(id, end_id, "end");
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                self.emit_node(id, &format!("ArrayLiteral{}", type_info), "lightyellow");
                for (i, e) in exprs.iter().enumerate() {
                    let e_id = self.visit_expr(e);
                    self.emit_edge(id, e_id, &format!("elem_{}", i));
                }
            }
            AnalyzedExprKind::Some(e) => {
                self.emit_node(id, &format!("Some{}", type_info), "lightyellow");
                let e_id = self.visit_expr(e);
                self.emit_edge(id, e_id, "value");
            }
            AnalyzedExprKind::None => {
                self.emit_node(id, &format!("None{}", type_info), "lightyellow");
            }
            AnalyzedExprKind::Ok(e) => {
                self.emit_node(id, &format!("Ok{}", type_info), "lightyellow");
                let e_id = self.visit_expr(e);
                self.emit_edge(id, e_id, "value");
            }
            AnalyzedExprKind::Err(e) => {
                self.emit_node(id, &format!("Err{}", type_info), "lightyellow");
                let e_id = self.visit_expr(e);
                self.emit_edge(id, e_id, "error");
            }
            AnalyzedExprKind::Unwrap(e) => {
                self.emit_node(id, &format!("Unwrap{}", type_info), "lightyellow");
                let e_id = self.visit_expr(e);
                self.emit_edge(id, e_id, "target");
            }
            AnalyzedExprKind::Try(e) => {
                self.emit_node(id, &format!("Try{}", type_info), "lightyellow");
                let e_id = self.visit_expr(e);
                self.emit_edge(id, e_id, "target");
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.emit_node(id, &format!("IndexAccess{}", type_info), "lightyellow");
                let array_id = self.visit_expr(array);
                let index_id = self.visit_expr(index);
                self.emit_edge(id, array_id, "array");
                self.emit_edge(id, index_id, "index");
            }
            AnalyzedExprKind::FunctionCall { func, args } => {
                self.emit_node(
                    id,
                    &format!("FunctionCall\\n{}{}", func, type_info),
                    "lightyellow",
                );
                for (i, arg) in args.iter().enumerate() {
                    let arg_id = self.visit_expr(arg);
                    self.emit_edge(id, arg_id, &format!("arg_{}", i));
                }
            }
            AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.emit_node(
                    id,
                    &format!("MethodCall\\n.{}{}", method, type_info),
                    "lightyellow",
                );
                let rec_id = self.visit_expr(receiver);
                self.emit_edge(id, rec_id, "receiver");
                for (i, arg) in args.iter().enumerate() {
                    let arg_id = self.visit_expr(arg);
                    self.emit_edge(id, arg_id, &format!("arg_{}", i));
                }
            }
            AnalyzedExprKind::StructInstantiation {
                type_name,
                fields,
                args,
            } => {
                self.emit_node(
                    id,
                    &format!("StructInstantiation\\n{}{}", type_name, type_info),
                    "lightyellow",
                );
                for (i, arg) in args.iter().enumerate() {
                    let arg_id = self.visit_expr(arg);
                    let field_name = fields.get(i).map(|s| s.as_str()).unwrap_or("?");
                    self.emit_edge(id, arg_id, field_name);
                }
            }
            AnalyzedExprKind::Lambda {
                params,
                body,
                capture_mode,
            } => {
                let capture_str = match capture_mode {
                    crate::semantic::CaptureMode::Borrow => "borrow",
                    crate::semantic::CaptureMode::Move => "move",
                };
                let params_str = params.join(", ");
                self.emit_node(
                    id,
                    &format!("Lambda\\n[{}] |{}|{}", capture_str, params_str, type_info),
                    "lightyellow",
                );
                let body_id = self.visit_expr(body);
                self.emit_edge(id, body_id, "body");
            }
            AnalyzedExprKind::CollectionNew { collection_type } => {
                self.emit_node(
                    id,
                    &format!("CollectionNew\\n{}::new(){}", collection_type, type_info),
                    "lightyellow",
                );
            }
            AnalyzedExprKind::Assert { condition } => {
                self.emit_node(id, &format!("Assert{}", type_info), "lightyellow");
                let cond_id = self.visit_expr(condition);
                self.emit_edge(id, cond_id, "condition");
            }
            AnalyzedExprKind::AssertEq { left, right } => {
                self.emit_node(id, &format!("AssertEq{}", type_info), "lightyellow");
                let left_id = self.visit_expr(left);
                let right_id = self.visit_expr(right);
                self.emit_edge(id, left_id, "left");
                self.emit_edge(id, right_id, "right");
            }
        }

        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{GlossaType, Scope};

    #[test]
    fn test_dot_generator_coverage() {
        let scope = Scope::new();

        let mut statements = Vec::new();

        // Binding
        statements.push(AnalyzedStatement::Binding {
            name: "test".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(42),
                glossa_type: GlossaType::Number,
            },
            mutable: true,
        });

        // Assignment
        statements.push(AnalyzedStatement::Assignment {
            name: "test".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("val".into()),
                glossa_type: GlossaType::String,
            },
        });

        // Print & Query & Expression
        let dummy_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        };
        statements.push(AnalyzedStatement::Print(vec![dummy_expr.clone()]));
        statements.push(AnalyzedStatement::Query(vec![dummy_expr.clone()]));
        statements.push(AnalyzedStatement::Expression(vec![dummy_expr.clone()]));

        // If with else
        statements.push(AnalyzedStatement::If {
            condition: Box::new(dummy_expr.clone()),
            then_body: vec![AnalyzedStatement::Break],
            else_body: Some(vec![AnalyzedStatement::Continue]),
        });

        // While & For
        statements.push(AnalyzedStatement::While {
            condition: Box::new(dummy_expr.clone()),
            body: vec![AnalyzedStatement::Break],
        });
        statements.push(AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(dummy_expr.clone()),
            body: vec![AnalyzedStatement::Break],
        });

        // Match
        statements.push(AnalyzedStatement::Match {
            scrutinee: Box::new(dummy_expr.clone()),
            arms: vec![(dummy_expr.clone(), vec![AnalyzedStatement::Break])],
        });

        // Return
        statements.push(AnalyzedStatement::Return {
            value: Some(Box::new(dummy_expr.clone())),
        });

        // FunctionDef
        statements.push(AnalyzedStatement::FunctionDef {
            name: "func".into(),
            params: vec![("p".into(), Some(GlossaType::Number))],
            body: vec![],
            return_type: Some(GlossaType::Number),
        });

        // TypeDef
        statements.push(AnalyzedStatement::TypeDefinition {
            name: "Type".into(),
            fields: vec![("f".into(), GlossaType::Number)],
        });

        // TraitDef & TraitImpl
        let method = crate::semantic::AnalyzedMethod {
            name: "meth".into(),
            params: vec![],
            body: Some(vec![]),
            return_type: None,
        };
        statements.push(AnalyzedStatement::TraitDefinition {
            name: "Trait".into(),
            methods: vec![method.clone()],
        });
        statements.push(AnalyzedStatement::TraitImplementation {
            trait_name: "Trait".into(),
            type_name: "Type".into(),
            methods: vec![method],
        });

        // TestDeclaration
        statements.push(AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![],
        });

        // Add all AnalyzedExprKind variants wrapped in an Expression statement
        let exprs = vec![
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("var".into()),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(dummy_expr.clone()),
                    property: "prop".into(),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::VerbCall {
                    verb: "verb".into(),
                    args: vec![dummy_expr.clone()],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(dummy_expr.clone()),
                    op: crate::morphology::lexicon::BinaryOp::Add,
                    right: Box::new(dummy_expr.clone()),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::UnaryOp {
                    op: crate::morphology::lexicon::UnaryOp::Not,
                    operand: Box::new(dummy_expr.clone()),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Range {
                    start: Box::new(dummy_expr.clone()),
                    end: Box::new(dummy_expr.clone()),
                    inclusive: true,
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Range {
                    start: Box::new(dummy_expr.clone()),
                    end: Box::new(dummy_expr.clone()),
                    inclusive: false,
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![dummy_expr.clone()]),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Some(Box::new(dummy_expr.clone())),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Ok(Box::new(dummy_expr.clone())),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Err(Box::new(dummy_expr.clone())),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(dummy_expr.clone())),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Try(Box::new(dummy_expr.clone())),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(dummy_expr.clone()),
                    index: Box::new(dummy_expr.clone()),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::FunctionCall {
                    func: "func".into(),
                    args: vec![dummy_expr.clone()],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(dummy_expr.clone()),
                    method: "meth".into(),
                    args: vec![dummy_expr.clone()],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::StructInstantiation {
                    type_name: "Type".into(),
                    fields: vec!["f".into()],
                    args: vec![dummy_expr.clone()],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Lambda {
                    params: vec!["p".into()],
                    body: Box::new(dummy_expr.clone()),
                    capture_mode: crate::semantic::CaptureMode::Move,
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::CollectionNew {
                    collection_type: "List".into(),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Assert {
                    condition: Box::new(dummy_expr.clone()),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::AssertEq {
                    left: Box::new(dummy_expr.clone()),
                    right: Box::new(dummy_expr.clone()),
                },
                glossa_type: GlossaType::Unknown,
            },
        ];

        statements.push(AnalyzedStatement::Expression(exprs));

        let program = AnalyzedProgram {
            statements,
            scope,
        };

        let mut generator = DotGenerator::new();
        let dot = generator.generate(&program);

        assert!(dot.contains("digraph AST {"));
        assert!(dot.contains("Binding\\nmut test"));
        assert!(dot.contains("Number\\n42"));
        assert!(dot.contains("Assignment\\ntest"));
        assert!(dot.contains("Match"));
        assert!(dot.contains("StructInstantiation"));
    }
}
