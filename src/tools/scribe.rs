//! The Scribe (ὁ Γραμματεύς) - JSON AST Exporter
//!
//! This module implements "The Scribe" tool, which takes a parsed and analyzed
//! ΓΛΩΣΣΑ program and exports its semantic Abstract Syntax Tree (AST) to a
//! structured JSON format.
//!
//! # Purpose
//!
//! The Scribe serves as an integration point for external tools (like web IDEs,
//! alternative backends, or static analyzers written in other languages) by
//! providing a standard, machine-readable serialization of the compiler's internal state.
//!
//! By manually constructing the JSON string, we avoid polluting the core `ast` and `semantic`
//! structs with `serde` derive macros, preserving the "additive only" architecture.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Runs the Scribe tool to export the AST to JSON.
pub fn run_scribe(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Γραμματεύς (Exporting JSON AST)", "✍️");

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

    let json_output = program_to_json(&program);

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S C R I B E".bold().cyan());
    println!("   {}", "JSON AST Export".italic().dim());
    println!();
    println!("{}", json_output);

    Ok(())
}

fn escape_json_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('"');
    for c in s.chars() {
        match c {
            '"' => escaped.push_str(r#"\""#),
            '\\' => escaped.push_str(r#"\\"#),
            '\n' => escaped.push_str(r#"\n"#),
            '\r' => escaped.push_str(r#"\r"#),
            '\t' => escaped.push_str(r#"\t"#),
            _ => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}

/// Converts an AnalyzedProgram to a JSON string
pub fn program_to_json(program: &AnalyzedProgram) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str("{\n  \"type\": \"Program\",\n  \"statements\": [\n");

    for (i, stmt) in program.statements.iter().enumerate() {
        out.push_str(&statement_to_json(stmt, 4));
        if i < program.statements.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}");
    out
}

fn indent(level: usize) -> String {
    " ".repeat(level)
}

fn statement_to_json(stmt: &AnalyzedStatement, lvl: usize) -> String {
    let ind = indent(lvl);
    let ind2 = indent(lvl + 2);
    let mut out = String::new();

    out.push_str(&format!("{}{{\n", ind));

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            out.push_str(&format!("{}\"type\": \"Binding\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {},\n",
                ind2,
                escape_json_string(name.as_str())
            ));
            out.push_str(&format!("{}\"mutable\": {},\n", ind2, mutable));
            out.push_str(&format!(
                "{}\"value\": \n{}",
                ind2,
                expr_to_json(value, lvl + 2)
            ));
        }
        AnalyzedStatement::Assignment { name, value } => {
            out.push_str(&format!("{}\"type\": \"Assignment\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {},\n",
                ind2,
                escape_json_string(name.as_str())
            ));
            out.push_str(&format!(
                "{}\"value\": \n{}",
                ind2,
                expr_to_json(value, lvl + 2)
            ));
        }
        AnalyzedStatement::Print(exprs) => {
            out.push_str(&format!("{}\"type\": \"Print\",\n", ind2));
            out.push_str(&format!("{}\"expressions\": [\n", ind2));
            for (i, e) in exprs.iter().enumerate() {
                out.push_str(&expr_to_json(e, lvl + 4));
                if i < exprs.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedStatement::Return { value } => {
            out.push_str(&format!("{}\"type\": \"Return\",\n", ind2));
            if let Some(expr) = value {
                out.push_str(&format!(
                    "{}\"expression\": \n{}",
                    ind2,
                    expr_to_json(expr, lvl + 2)
                ));
            } else {
                out.push_str(&format!("{}\"expression\": null", ind2));
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            out.push_str(&format!("{}\"type\": \"If\",\n", ind2));
            out.push_str(&format!(
                "{}\"condition\": \n{},\n",
                ind2,
                expr_to_json(condition, lvl + 2)
            ));

            out.push_str(&format!("{}\"then_branch\": [\n", ind2));
            for (i, s) in then_body.iter().enumerate() {
                out.push_str(&statement_to_json(s, lvl + 4));
                if i < then_body.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}],\n", ind2));

            out.push_str(&format!("{}\"else_branch\": [\n", ind2));
            if let Some(els) = else_body {
                for (i, s) in els.iter().enumerate() {
                    out.push_str(&statement_to_json(s, lvl + 4));
                    if i < els.len() - 1 {
                        out.push(',');
                    }
                    out.push('\n');
                }
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedStatement::While { condition, body } => {
            out.push_str(&format!("{}\"type\": \"While\",\n", ind2));
            out.push_str(&format!(
                "{}\"condition\": \n{},\n",
                ind2,
                expr_to_json(condition, lvl + 2)
            ));
            out.push_str(&format!("{}\"body\": [\n", ind2));
            for (i, s) in body.iter().enumerate() {
                out.push_str(&statement_to_json(s, lvl + 4));
                if i < body.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            out.push_str(&format!("{}\"type\": \"For\",\n", ind2));
            out.push_str(&format!(
                "{}\"item\": {},\n",
                ind2,
                escape_json_string(variable.as_str())
            ));
            out.push_str(&format!(
                "{}\"iterable\": \n{},\n",
                ind2,
                expr_to_json(iterator, lvl + 2)
            ));
            out.push_str(&format!("{}\"body\": [\n", ind2));
            for (i, s) in body.iter().enumerate() {
                out.push_str(&statement_to_json(s, lvl + 4));
                if i < body.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedStatement::Break => {
            out.push_str(&format!("{}\"type\": \"Break\"", ind2));
        }
        AnalyzedStatement::Continue => {
            out.push_str(&format!("{}\"type\": \"Continue\"", ind2));
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            out.push_str(&format!("{}\"type\": \"TypeDefinition\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {},\n",
                ind2,
                escape_json_string(name.as_str())
            ));
            out.push_str(&format!("{}\"fields\": {{\n", ind2));
            for (i, (fname, ftype)) in fields.iter().enumerate() {
                out.push_str(&format!(
                    "  {} {}: {}",
                    ind2,
                    escape_json_string(fname.as_str()),
                    escape_json_string(&format!("{:?}", ftype))
                ));
                if i < fields.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}}}", ind2));
        }
        AnalyzedStatement::TraitDefinition { name, methods: _ } => {
            out.push_str(&format!("{}\"type\": \"TraitDefinition\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {}",
                ind2,
                escape_json_string(name.as_str())
            ));
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods: _,
        } => {
            out.push_str(&format!("{}\"type\": \"TraitImplementation\",\n", ind2));
            out.push_str(&format!(
                "{}\"trait_name\": {},\n",
                ind2,
                escape_json_string(trait_name.as_str())
            ));
            out.push_str(&format!(
                "{}\"struct_name\": {}",
                ind2,
                escape_json_string(type_name.as_str())
            ));
        }
        AnalyzedStatement::FunctionDef {
            name,
            params: _,
            body,
            return_type: _,
            ..
        } => {
            out.push_str(&format!("{}\"type\": \"FunctionDef\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {},\n",
                ind2,
                escape_json_string(name.as_str())
            ));
            out.push_str(&format!("{}\"body\": [\n", ind2));
            for (i, s) in body.iter().enumerate() {
                out.push_str(&statement_to_json(s, lvl + 4));
                if i < body.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            out.push_str(&format!("{}\"type\": \"TestDeclaration\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {},\n",
                ind2,
                escape_json_string(name.as_str())
            ));
            out.push_str(&format!("{}\"body\": [\n", ind2));
            for (i, s) in body.iter().enumerate() {
                out.push_str(&statement_to_json(s, lvl + 4));
                if i < body.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedStatement::Expression(exprs) => {
            out.push_str(&format!("{}\"type\": \"Expression\",\n", ind2));
            if let Some(expr) = exprs.first() {
                out.push_str(&format!(
                    "{}\"expression\": \n{}",
                    ind2,
                    expr_to_json(expr, lvl + 2)
                ));
            } else {
                out.push_str(&format!("{}\"expression\": null", ind2));
            }
        }
        _ => {
            out.push_str(&format!("{}\"type\": \"Unknown\"", ind2));
        }
    }

    out.push_str(&format!("\n{}}}", ind));
    out
}

fn expr_to_json(expr: &AnalyzedExpr, lvl: usize) -> String {
    let ind = indent(lvl);
    let ind2 = indent(lvl + 2);
    let mut out = String::new();

    out.push_str(&format!("{}{{\n", ind));
    // simplified type representation
    out.push_str(&format!(
        "{}\"glossa_type\": {},\n",
        ind2,
        escape_json_string(&format!("{:?}", expr.glossa_type))
    ));

    match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => {
            out.push_str(&format!("{}\"kind\": \"NumberLiteral\",\n", ind2));
            out.push_str(&format!("{}\"value\": {}", ind2, n));
        }
        AnalyzedExprKind::StringLiteral(s) => {
            out.push_str(&format!("{}\"kind\": \"StringLiteral\",\n", ind2));
            out.push_str(&format!("{}\"value\": {}", ind2, escape_json_string(s)));
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            out.push_str(&format!("{}\"kind\": \"BooleanLiteral\",\n", ind2));
            out.push_str(&format!("{}\"value\": {}", ind2, b));
        }
        AnalyzedExprKind::Variable(v) => {
            out.push_str(&format!("{}\"kind\": \"Variable\",\n", ind2));
            out.push_str(&format!(
                "{}\"value\": {}",
                ind2,
                escape_json_string(v.as_str())
            ));
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            out.push_str(&format!("{}\"kind\": \"ArrayLiteral\",\n", ind2));
            out.push_str(&format!("{}\"elements\": [\n", ind2));
            for (i, e) in exprs.iter().enumerate() {
                out.push_str(&expr_to_json(e, lvl + 4));
                if i < exprs.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}]", ind2));
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            out.push_str(&format!("{}\"kind\": \"BinOp\",\n", ind2));
            out.push_str(&format!(
                "{}\"op\": {},\n",
                ind2,
                escape_json_string(&format!("{:?}", op))
            ));
            out.push_str(&format!(
                "{}\"left\": \n{},\n",
                ind2,
                expr_to_json(left, lvl + 2)
            ));
            out.push_str(&format!(
                "{}\"right\": \n{}",
                ind2,
                expr_to_json(right, lvl + 2)
            ));
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            out.push_str(&format!("{}\"kind\": \"UnaryOp\",\n", ind2));
            out.push_str(&format!(
                "{}\"op\": {},\n",
                ind2,
                escape_json_string(&format!("{:?}", op))
            ));
            out.push_str(&format!(
                "{}\"operand\": \n{}",
                ind2,
                expr_to_json(operand, lvl + 2)
            ));
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            out.push_str(&format!("{}\"kind\": \"PropertyAccess\",\n", ind2));
            out.push_str(&format!(
                "{}\"property\": {},\n",
                ind2,
                escape_json_string(property.as_str())
            ));
            out.push_str(&format!(
                "{}\"owner\": \n{}",
                ind2,
                expr_to_json(owner, lvl + 2)
            ));
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args: _,
        } => {
            out.push_str(&format!("{}\"kind\": \"MethodCall\",\n", ind2));
            out.push_str(&format!(
                "{}\"method\": {},\n",
                ind2,
                escape_json_string(method.as_str())
            ));
            out.push_str(&format!(
                "{}\"owner\": \n{}\n",
                ind2,
                expr_to_json(receiver, lvl + 2)
            ));
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            args: _,
            fields: _,
        } => {
            out.push_str(&format!("{}\"kind\": \"StructInstantiation\",\n", ind2));
            out.push_str(&format!(
                "{}\"name\": {}\n",
                ind2,
                escape_json_string(type_name.as_str())
            ));
        }
        AnalyzedExprKind::VerbCall { verb, args: _ } => {
            out.push_str(&format!("{}\"kind\": \"VerbCall\",\n", ind2));
            out.push_str(&format!(
                "{}\"verb\": {}\n",
                ind2,
                escape_json_string(verb.as_str())
            ));
        }
        _ => {
            out.push_str(&format!("{}\"kind\": \"Other\"", ind2));
        }
    }

    out.push_str(&format!("\n{}}}", ind));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]


    #[test]
    fn test_scribe_basic() {
        let source = "ξ πέντε ἔστω.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let json = program_to_json(&program);

        assert!(
            json.contains(r#""type": "Program""#),
            "Expected JSON to contain Program type"
        );
        assert!(
            json.contains(r#""name": "ξ""#),
            "Expected JSON to contain variable name"
        );
    }
}
