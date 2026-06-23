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
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::io::IsTerminal;
use std::path::Path;

/// Runs the Haruspex tool on a given Glossa source file.
///
/// This function reads the provided source file, parses and semantically analyzes it,
/// and then prints a Graphviz DOT format representation of the Abstract Syntax Tree (AST)
/// to the standard output.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::haruspex::run_haruspex;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// if let Err(e) = run_haruspex(&input) {
///     eprintln!("Haruspex failed: {}", e);
/// }
/// ```
///
/// # Errors
///
/// Returns a [`miette::Result`] if the file cannot be read, or if there is a parsing
/// or semantic error during compilation.
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

    let dot = generate_dot(&program);

    print_dashboard(&dot, std::io::stdout().is_terminal());
    Ok(())
}

fn print_dashboard(dot: &str, is_tty: bool) {
    if is_tty {
        println!();
        println!("   {}", "Γ Λ Ω Σ Σ Α   H A R U S P E X".cyan().bold());
        println!(
            "   {}",
            "AST Graphviz DOT Representation Generated".italic().dim()
        );
        println!();

        let actual_nodes = dot.lines().filter(|l| l.contains("[label=")).count();
        let edges = dot.matches("->").count();

        println!("   {} {}", "Nodes:".bold(), actual_nodes.to_string().cyan());
        println!("   {} {}", "Edges:".bold(), edges.to_string().cyan());
        println!();
        println!(
            "   {}",
            "To view the graph, pipe this command to a file or tool:".dim()
        );
        println!(
            "   {}",
            "cargo run --features nova --bin glossa -- haruspex <file> > ast.dot".dim()
        );
        println!(
            "   {}",
            "cargo run --features nova --bin glossa -- haruspex <file> | dot -Tpng > ast.png".dim()
        );
        println!();
    } else {
        println!("{}", dot);
    }
}

fn generate_dot(program: &AnalyzedProgram) -> String {
    let mut next_id = 0;
    let mut output = String::new();
    output.push_str("digraph AST {\n");
    output.push_str(
        "    node [shape=box, style=filled, fillcolor=lightgrey, fontname=\"Courier\"];\n",
    );
    output.push_str("    edge [fontname=\"Courier\"];\n");

    let root_id = get_next_id(&mut next_id);
    emit_node(&mut output, root_id, "Program", "lightblue");

    for stmt in &program.statements {
        let stmt_id = visit_statement(&mut next_id, &mut output, stmt);
        emit_edge(&mut output, root_id, stmt_id, "");
    }

    output.push_str("}\n");
    output
}

fn get_next_id(next_id: &mut usize) -> usize {
    let id = *next_id;
    *next_id += 1;
    id
}

fn emit_node(output: &mut String, id: usize, label: &str, color: &str) {
    // Escape quotes
    let safe_label = label.replace("\"", "\\\"");
    let _ = writeln!(
        output,
        "    node_{} [label=\"{}\", fillcolor=\"{}\"];",
        id, safe_label, color
    );
}

fn emit_edge(output: &mut String, from: usize, to: usize, label: &str) {
    if label.is_empty() {
        let _ = writeln!(output, "    node_{} -> node_{};", from, to);
    } else {
        let safe_label = label.replace("\"", "\\\"");
        let _ = writeln!(
            output,
            "    node_{} -> node_{} [label=\"{}\"];",
            from, to, safe_label
        );
    }
}

fn visit_statement(next_id: &mut usize, output: &mut String, stmt: &AnalyzedStatement) -> usize {
    let id = get_next_id(next_id);

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let mut_str = if *mutable { "mut " } else { "" };
            emit_node(
                output,
                id,
                &format!("Binding\\n{}{}", mut_str, name),
                "lightgreen",
            );
            let val_id = visit_expr(next_id, output, value);
            emit_edge(output, id, val_id, "value");
        }
        AnalyzedStatement::Assignment { name, value } => {
            emit_node(output, id, &format!("Assignment\\n{}", name), "lightgreen");
            let val_id = visit_expr(next_id, output, value);
            emit_edge(output, id, val_id, "value");
        }
        AnalyzedStatement::Print(exprs) => {
            emit_node(output, id, "Print", "lightgreen");
            for (i, expr) in exprs.iter().enumerate() {
                let child_id = visit_expr(next_id, output, expr);
                emit_edge(output, id, child_id, &format!("arg_{}", i));
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            emit_node(output, id, "ExpressionStmt", "lightgreen");
            for (i, expr) in exprs.iter().enumerate() {
                let child_id = visit_expr(next_id, output, expr);
                emit_edge(output, id, child_id, &format!("expr_{}", i));
            }
        }
        AnalyzedStatement::Query(exprs) => {
            emit_node(output, id, "Query", "lightgreen");
            for (i, expr) in exprs.iter().enumerate() {
                let child_id = visit_expr(next_id, output, expr);
                emit_edge(output, id, child_id, &format!("arg_{}", i));
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            visit_if_statement(
                next_id,
                output,
                id,
                condition,
                then_body,
                else_body.as_deref(),
            );
        }
        AnalyzedStatement::While { condition, body } => {
            visit_while_statement(next_id, output, id, condition, body);
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            visit_for_statement(next_id, output, id, variable, iterator, body);
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            visit_match_statement(next_id, output, id, scrutinee, arms);
        }
        AnalyzedStatement::Break => emit_node(output, id, "Break", "lightgreen"),
        AnalyzedStatement::Continue => emit_node(output, id, "Continue", "lightgreen"),
        AnalyzedStatement::Return { value } => {
            emit_node(output, id, "Return", "lightgreen");
            if let Some(val) = value {
                let val_id = visit_expr(next_id, output, val);
                emit_edge(output, id, val_id, "value");
            }
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
            visit_function_def_statement(next_id, output, id, name, params, body, return_type);
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            visit_type_def_statement(next_id, output, id, name, fields);
        }
        AnalyzedStatement::TraitDefinition { name, methods } => {
            visit_trait_def_statement(next_id, output, id, name, methods);
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => {
            visit_trait_impl_statement(next_id, output, id, trait_name, type_name, methods);
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            visit_test_decl_statement(next_id, output, id, name, body);
        }
    }

    id
}

fn visit_expr(next_id: &mut usize, output: &mut String, expr: &AnalyzedExpr) -> usize {
    let id = get_next_id(next_id);

    // Include type information in expression nodes
    let type_info = format!("\\n[{}]", expr.glossa_type);

    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            emit_node(
                output,
                id,
                &format!("String\\n\\\"{}\\\"{}", s, type_info),
                "lightyellow",
            );
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            emit_node(
                output,
                id,
                &format!("Number\\n{}{}", n, type_info),
                "lightyellow",
            );
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            emit_node(
                output,
                id,
                &format!("Boolean\\n{}{}", b, type_info),
                "lightyellow",
            );
        }
        AnalyzedExprKind::Variable(v) => {
            emit_node(
                output,
                id,
                &format!("Variable\\n{}{}", v, type_info),
                "lightyellow",
            );
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            emit_node(
                output,
                id,
                &format!("PropertyAccess\\n.{}{}", property, type_info),
                "lightyellow",
            );
            let owner_id = visit_expr(next_id, output, owner);
            emit_edge(output, id, owner_id, "owner");
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            visit_verb_call_expr(next_id, output, id, verb, args, &type_info);
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            visit_binop_expr(next_id, output, id, left, op, right, &type_info);
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            emit_node(
                output,
                id,
                &format!("UnaryOp\\n{:?}{}", op, type_info),
                "lightyellow",
            );
            let operand_id = visit_expr(next_id, output, operand);
            emit_edge(output, id, operand_id, "operand");
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            visit_range_expr(next_id, output, id, start, end, *inclusive, &type_info);
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            visit_array_literal_expr(next_id, output, id, exprs, &type_info);
        }
        AnalyzedExprKind::Some(e) => {
            emit_node(output, id, &format!("Some{}", type_info), "lightyellow");
            let e_id = visit_expr(next_id, output, e);
            emit_edge(output, id, e_id, "value");
        }
        AnalyzedExprKind::None => {
            emit_node(output, id, &format!("None{}", type_info), "lightyellow");
        }
        AnalyzedExprKind::Ok(e) => {
            emit_node(output, id, &format!("Ok{}", type_info), "lightyellow");
            let e_id = visit_expr(next_id, output, e);
            emit_edge(output, id, e_id, "value");
        }
        AnalyzedExprKind::Err(e) => {
            emit_node(output, id, &format!("Err{}", type_info), "lightyellow");
            let e_id = visit_expr(next_id, output, e);
            emit_edge(output, id, e_id, "error");
        }
        AnalyzedExprKind::Unwrap(e) => {
            emit_node(output, id, &format!("Unwrap{}", type_info), "lightyellow");
            let e_id = visit_expr(next_id, output, e);
            emit_edge(output, id, e_id, "target");
        }
        AnalyzedExprKind::Try(e) => {
            emit_node(output, id, &format!("Try{}", type_info), "lightyellow");
            let e_id = visit_expr(next_id, output, e);
            emit_edge(output, id, e_id, "target");
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            emit_node(
                output,
                id,
                &format!("IndexAccess{}", type_info),
                "lightyellow",
            );
            let array_id = visit_expr(next_id, output, array);
            let index_id = visit_expr(next_id, output, index);
            emit_edge(output, id, array_id, "array");
            emit_edge(output, id, index_id, "index");
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            visit_function_call_expr(next_id, output, id, func, args, &type_info);
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            visit_method_call_expr(next_id, output, id, receiver, method, args, &type_info);
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            visit_struct_instantiation_expr(
                next_id, output, id, type_name, fields, args, &type_info,
            );
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            visit_lambda_expr(next_id, output, id, params, body, capture_mode, &type_info);
        }
        AnalyzedExprKind::CollectionNew { collection_type } => {
            emit_node(
                output,
                id,
                &format!("CollectionNew\\n{}::new(){}", collection_type, type_info),
                "lightyellow",
            );
        }
        AnalyzedExprKind::Assert { condition } => {
            emit_node(output, id, &format!("Assert{}", type_info), "lightyellow");
            let cond_id = visit_expr(next_id, output, condition);
            emit_edge(output, id, cond_id, "condition");
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            emit_node(output, id, &format!("AssertEq{}", type_info), "lightyellow");
            let left_id = visit_expr(next_id, output, left);
            let right_id = visit_expr(next_id, output, right);
            emit_edge(output, id, left_id, "left");
            emit_edge(output, id, right_id, "right");
        }
    }

    id
}

fn visit_if_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: Option<&[AnalyzedStatement]>,
) {
    emit_node(output, id, "If", "lightgreen");
    let cond_id = visit_expr(next_id, output, condition);
    emit_edge(output, id, cond_id, "condition");

    let then_id = get_next_id(next_id);
    emit_node(output, then_id, "Block (Then)", "white");
    emit_edge(output, id, then_id, "then");
    for s in then_body {
        let s_id = visit_statement(next_id, output, s);
        emit_edge(output, then_id, s_id, "");
    }

    if let Some(else_b) = else_body {
        let else_id = get_next_id(next_id);
        emit_node(output, else_id, "Block (Else)", "white");
        emit_edge(output, id, else_id, "else");
        for s in else_b {
            let s_id = visit_statement(next_id, output, s);
            emit_edge(output, else_id, s_id, "");
        }
    }
}

fn visit_while_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    condition: &AnalyzedExpr,
    body: &[AnalyzedStatement],
) {
    emit_node(output, id, "While", "lightgreen");
    let cond_id = visit_expr(next_id, output, condition);
    emit_edge(output, id, cond_id, "condition");

    let body_id = get_next_id(next_id);
    emit_node(output, body_id, "Block", "white");
    emit_edge(output, id, body_id, "body");
    for s in body {
        let s_id = visit_statement(next_id, output, s);
        emit_edge(output, body_id, s_id, "");
    }
}

fn visit_for_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    variable: &str,
    iterator: &AnalyzedExpr,
    body: &[AnalyzedStatement],
) {
    emit_node(output, id, &format!("For\\n{}", variable), "lightgreen");
    let iter_id = visit_expr(next_id, output, iterator);
    emit_edge(output, id, iter_id, "iterator");

    let body_id = get_next_id(next_id);
    emit_node(output, body_id, "Block", "white");
    emit_edge(output, id, body_id, "body");
    for s in body {
        let s_id = visit_statement(next_id, output, s);
        emit_edge(output, body_id, s_id, "");
    }
}

fn visit_match_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    scrutinee: &AnalyzedExpr,
    arms: &[(AnalyzedExpr, Vec<AnalyzedStatement>)],
) {
    emit_node(output, id, "Match", "lightgreen");
    let scrutinee_id = visit_expr(next_id, output, scrutinee);
    emit_edge(output, id, scrutinee_id, "scrutinee");

    for (i, (pat, body)) in arms.iter().enumerate() {
        let arm_id = get_next_id(next_id);
        emit_node(output, arm_id, &format!("Arm_{}", i), "white");
        emit_edge(output, id, arm_id, "");

        let pat_id = visit_expr(next_id, output, pat);
        emit_edge(output, arm_id, pat_id, "pattern");

        for s in body {
            let s_id = visit_statement(next_id, output, s);
            emit_edge(output, arm_id, s_id, "body");
        }
    }
}

fn visit_function_def_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    name: &str,
    params: &[(smol_str::SmolStr, Option<crate::semantic::GlossaType>)],
    body: &[AnalyzedStatement],
    return_type: &Option<crate::semantic::GlossaType>,
) {
    let ret_str = return_type
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "()".to_string());
    emit_node(
        output,
        id,
        &format!("FunctionDef\\n{} -> {}", name, ret_str),
        "lightgreen",
    );

    for (i, (p_name, p_type)) in params.iter().enumerate() {
        let type_str = p_type
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_else(|| "?".to_string());
        let p_id = get_next_id(next_id);
        emit_node(
            output,
            p_id,
            &format!("{}: {}", p_name, type_str),
            "lightyellow",
        );
        emit_edge(output, id, p_id, &format!("param_{}", i));
    }

    let body_id = get_next_id(next_id);
    emit_node(output, body_id, "Block", "white");
    emit_edge(output, id, body_id, "body");
    for s in body {
        let s_id = visit_statement(next_id, output, s);
        emit_edge(output, body_id, s_id, "");
    }
}

fn visit_type_def_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    name: &str,
    fields: &[(smol_str::SmolStr, crate::semantic::GlossaType)],
) {
    emit_node(
        output,
        id,
        &format!("TypeDefinition\\n{}", name),
        "lightgreen",
    );
    for (i, (f_name, f_type)) in fields.iter().enumerate() {
        let f_id = get_next_id(next_id);
        emit_node(
            output,
            f_id,
            &format!("{}: {}", f_name, f_type),
            "lightyellow",
        );
        emit_edge(output, id, f_id, &format!("field_{}", i));
    }
}

fn visit_trait_def_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    name: &str,
    methods: &[crate::semantic::AnalyzedMethod],
) {
    emit_node(
        output,
        id,
        &format!("TraitDefinition\\n{}", name),
        "lightgreen",
    );
    for (i, method) in methods.iter().enumerate() {
        let m_id = get_next_id(next_id);
        emit_node(
            output,
            m_id,
            &format!("MethodDecl\\n{}", method.name),
            "lightyellow",
        );
        emit_edge(output, id, m_id, &format!("method_{}", i));
    }
}

fn visit_trait_impl_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    trait_name: &str,
    type_name: &str,
    methods: &[crate::semantic::AnalyzedMethod],
) {
    emit_node(
        output,
        id,
        &format!("TraitImpl\\n{} for {}", trait_name, type_name),
        "lightgreen",
    );
    for (i, method) in methods.iter().enumerate() {
        let m_id = get_next_id(next_id);
        emit_node(
            output,
            m_id,
            &format!("MethodImpl\\n{}", method.name),
            "lightyellow",
        );
        emit_edge(output, id, m_id, &format!("method_{}", i));
    }
}

fn visit_test_decl_statement(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    name: &str,
    body: &[AnalyzedStatement],
) {
    emit_node(
        output,
        id,
        &format!("TestDeclaration\\n{}", name),
        "lightgreen",
    );
    let body_id = get_next_id(next_id);
    emit_node(output, body_id, "Block", "white");
    emit_edge(output, id, body_id, "body");
    for s in body {
        let s_id = visit_statement(next_id, output, s);
        emit_edge(output, body_id, s_id, "");
    }
}

fn visit_verb_call_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    verb: &str,
    args: &[AnalyzedExpr],
    type_info: &str,
) {
    emit_node(
        output,
        id,
        &format!("VerbCall\\n{}{}", verb, type_info),
        "lightyellow",
    );
    for (i, arg) in args.iter().enumerate() {
        let arg_id = visit_expr(next_id, output, arg);
        emit_edge(output, id, arg_id, &format!("arg_{}", i));
    }
}

fn visit_binop_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    left: &AnalyzedExpr,
    op: &crate::morphology::BinaryOp,
    right: &AnalyzedExpr,
    type_info: &str,
) {
    emit_node(
        output,
        id,
        &format!("BinOp\\n{:?}{}", op, type_info),
        "lightyellow",
    );
    let left_id = visit_expr(next_id, output, left);
    let right_id = visit_expr(next_id, output, right);
    emit_edge(output, id, left_id, "left");
    emit_edge(output, id, right_id, "right");
}

fn visit_range_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    start: &AnalyzedExpr,
    end: &AnalyzedExpr,
    inclusive: bool,
    type_info: &str,
) {
    let range_sym = if inclusive { "..=" } else { ".." };
    emit_node(
        output,
        id,
        &format!("Range\\n{}{}", range_sym, type_info),
        "lightyellow",
    );
    let start_id = visit_expr(next_id, output, start);
    let end_id = visit_expr(next_id, output, end);
    emit_edge(output, id, start_id, "start");
    emit_edge(output, id, end_id, "end");
}

fn visit_array_literal_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    exprs: &[AnalyzedExpr],
    type_info: &str,
) {
    emit_node(
        output,
        id,
        &format!("ArrayLiteral{}", type_info),
        "lightyellow",
    );
    for (i, e) in exprs.iter().enumerate() {
        let e_id = visit_expr(next_id, output, e);
        emit_edge(output, id, e_id, &format!("elem_{}", i));
    }
}

fn visit_function_call_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    func: &str,
    args: &[AnalyzedExpr],
    type_info: &str,
) {
    emit_node(
        output,
        id,
        &format!("FunctionCall\\n{}{}", func, type_info),
        "lightyellow",
    );
    for (i, arg) in args.iter().enumerate() {
        let arg_id = visit_expr(next_id, output, arg);
        emit_edge(output, id, arg_id, &format!("arg_{}", i));
    }
}

fn visit_method_call_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    receiver: &AnalyzedExpr,
    method: &str,
    args: &[AnalyzedExpr],
    type_info: &str,
) {
    emit_node(
        output,
        id,
        &format!("MethodCall\\n.{}{}", method, type_info),
        "lightyellow",
    );
    let rec_id = visit_expr(next_id, output, receiver);
    emit_edge(output, id, rec_id, "receiver");
    for (i, arg) in args.iter().enumerate() {
        let arg_id = visit_expr(next_id, output, arg);
        emit_edge(output, id, arg_id, &format!("arg_{}", i));
    }
}

fn visit_struct_instantiation_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    type_name: &str,
    fields: &[smol_str::SmolStr],
    args: &[AnalyzedExpr],
    type_info: &str,
) {
    emit_node(
        output,
        id,
        &format!("StructInstantiation\\n{}{}", type_name, type_info),
        "lightyellow",
    );
    for (i, arg) in args.iter().enumerate() {
        let arg_id = visit_expr(next_id, output, arg);
        let field_name = fields.get(i).map(|s| s.as_str()).unwrap_or("?");
        emit_edge(output, id, arg_id, field_name);
    }
}

fn visit_lambda_expr(
    next_id: &mut usize,
    output: &mut String,
    id: usize,
    params: &[smol_str::SmolStr],
    body: &AnalyzedExpr,
    capture_mode: &crate::semantic::CaptureMode,
    type_info: &str,
) {
    let capture_str = match capture_mode {
        crate::semantic::CaptureMode::Borrow => "borrow",
        crate::semantic::CaptureMode::Move => "move",
    };
    let params_str = params.join(", ");
    emit_node(
        output,
        id,
        &format!("Lambda\\n[{}] |{}|{}", capture_str, params_str, type_info),
        "lightyellow",
    );
    let body_id = visit_expr(next_id, output, body);
    emit_edge(output, id, body_id, "body");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{GlossaType, Scope};

    #[test]
    fn test_print_dashboard_tty() {
        // Just verify it doesn't panic
        print_dashboard(
            "digraph AST { node_0 [label=\"Program\"]; node_0 -> node_1; }",
            true,
        );
    }

    #[test]
    fn test_print_dashboard_no_tty() {
        print_dashboard("digraph AST { node_0 [label=\"Program\"]; }", false);
    }

    #[test]
    fn test_run_haruspex_success() {
        // Need to test the actual success path
        let temp_dir = tempfile::tempdir().unwrap();
        let valid_file = temp_dir.path().join("valid.γλ");
        std::fs::write(&valid_file, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();
        let result = run_haruspex(&valid_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_haruspex_errors() {
        // Test missing file
        let not_found = std::path::Path::new("does_not_exist.γλ");
        let result = run_haruspex(not_found);
        assert!(result.is_err());

        // Test syntax error parsing via temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_file = temp_dir.path().join("invalid.γλ");
        std::fs::write(&invalid_file, "this is not valid glossa").unwrap();
        let result = run_haruspex(&invalid_file);
        assert!(result.is_err());
    }

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
        statements.push(AnalyzedStatement::Return { value: None });

        // FunctionDef
        statements.push(AnalyzedStatement::FunctionDef {
            name: "func".into(),
            params: vec![("p".into(), Some(GlossaType::Number))],
            body: vec![],
            return_type: Some(GlossaType::Number),
        });
        statements.push(AnalyzedStatement::FunctionDef {
            name: "func2".into(),
            params: vec![("p".into(), None)],
            body: vec![],
            return_type: None,
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
                expr: AnalyzedExprKind::StringLiteral("val".into()),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Unknown,
            },
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
                expr: AnalyzedExprKind::Lambda {
                    params: vec!["p".into()],
                    body: Box::new(dummy_expr.clone()),
                    capture_mode: crate::semantic::CaptureMode::Borrow,
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

        let program = AnalyzedProgram { statements, scope };

        let dot = generate_dot(&program);

        assert!(dot.contains("digraph AST {"));
        assert!(dot.contains("Binding\\nmut test"));
        assert!(dot.contains("Number\\n42"));
        assert!(dot.contains("Assignment\\ntest"));
        assert!(dot.contains("Match"));
        assert!(dot.contains("StructInstantiation"));
    }
}
