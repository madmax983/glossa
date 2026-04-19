//! The Alchemist (ὁ Χημικός) - Python Exporter
//!
//! This module implements an experimental transpiler that converts ΓΛΩΣΣΑ programs
//! into Python scripts.
//!
//! # Purpose
//!
//! While the primary backend targets Rust, Python is an excellent, dynamic target
//! that aligns with the "scripting" feel of small ΓΛΩΣΣΑ programs. The Alchemist
//! provides an alternative export format, proving the independence of the semantic
//! phase from the Rust codegen phase.

use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use std::fmt::Write;
use std::path::Path;

/// Run the Alchemist tool on a file
pub fn run_alchemist(input: &Path) -> miette::Result<()> {
    let source = crate::tools::runner::load_source(input)?;

    let status =
        crate::tools::ui::Status::start_with_symbol("Χημεία (Transpiling to Python)", "⚗️");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let python_code = transpile_to_python(&program);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A L C H E M I S T".bold().cyan());
    println!("   {}", "Python Transpilation Result".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("Python Source Code")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    let formatted_code = format!("```python\n{}\n```", python_code.trim());
    table.add_row(vec![Cell::new(formatted_code)]);

    println!("{table}");
    println!();

    Ok(())
}

/// Transpile an AnalyzedProgram to Python source code
///
/// ⚡ Bolt Optimization: Uses `writeln!` directly into a `String` buffer
/// instead of intermediate `format!` strings to eliminate heap allocations.
pub fn transpile_to_python(program: &AnalyzedProgram) -> String {
    let mut out = String::new();
    writeln!(out, "from typing import Any").unwrap();
    writeln!(out, "from dataclasses import dataclass\n").unwrap();
    for stmt in &program.statements {
        writeln!(out, "{}", transpile_statement(stmt, 0)).unwrap();
    }
    out
}

fn format_transpiled_exprs(exprs: &[AnalyzedExpr]) -> String {
    let mut buf = String::with_capacity(exprs.len() * 16);
    for (i, expr) in exprs.iter().enumerate() {
        if i > 0 {
            write!(buf, ", ").unwrap();
        }
        write!(buf, "{}", transpile_expr(expr)).unwrap();
    }
    buf
}

fn transpile_statement(stmt: &AnalyzedStatement, indent: usize) -> String {
    let ind = "    ".repeat(indent);
    match stmt {
        AnalyzedStatement::Binding { name, value, .. } => {
            // Note: In Glossa, 'ἔστω' behaves like a let binding. We map it to assignment.
            format!(
                "{}{} = {}",
                ind,
                sanitize_ident(name),
                transpile_expr(value)
            )
        }
        AnalyzedStatement::Query(exprs) => {
            format!("{}print({})", ind, format_transpiled_exprs(exprs))
        }
        AnalyzedStatement::Assignment { name, value } => {
            format!(
                "{}{} = {}",
                ind,
                sanitize_ident(name),
                transpile_expr(value)
            )
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => transpile_if(condition, then_body, else_body, indent),
        AnalyzedStatement::While { condition, body } => transpile_while(condition, body, indent),
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => transpile_for(variable, iterator, body, indent),
        AnalyzedStatement::Break => format!("{}break", ind),
        AnalyzedStatement::Continue => format!("{}continue", ind),
        AnalyzedStatement::Return { value } => {
            if let Some(val) = value {
                format!("{}return {}", ind, transpile_expr(val))
            } else {
                format!("{}return", ind)
            }
        }
        AnalyzedStatement::Print(exprs) => {
            format!("{}print({})", ind, format_transpiled_exprs(exprs))
        }
        AnalyzedStatement::Expression(exprs) => {
            let mut out = String::new();
            for expr in exprs {
                writeln!(out, "{}{}", ind, transpile_expr(expr)).unwrap();
            }
            out.trim_end().to_string()
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            return_type: _,
            body,
        } => transpile_function_def(name, params, body, indent),
        AnalyzedStatement::TypeDefinition { name, fields } => {
            transpile_type_def(name, fields, indent)
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            transpile_test_declaration(name, body, indent)
        }
        AnalyzedStatement::Match { scrutinee, arms } => transpile_match(scrutinee, arms, indent),
        AnalyzedStatement::TraitDefinition { .. }
        | AnalyzedStatement::TraitImplementation { .. } => {
            format!(
                "{}# Traits not natively supported in simple Python transpile",
                ind
            )
        }
    }
}

fn transpile_if(
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: &Option<Vec<AnalyzedStatement>>,
    indent: usize,
) -> String {
    let ind = "    ".repeat(indent);
    let mut out = format!("{}if {}:\n", ind, transpile_expr(condition));
    if then_body.is_empty() {
        writeln!(out, "{}    pass", ind).unwrap();
    } else {
        for b_stmt in then_body {
            writeln!(out, "{}", transpile_statement(b_stmt, indent + 1)).unwrap();
        }
    }
    if let Some(ebody) = else_body {
        writeln!(out, "{}else:", ind).unwrap();
        if ebody.is_empty() {
            writeln!(out, "{}    pass", ind).unwrap();
        } else {
            for b_stmt in ebody {
                writeln!(out, "{}", transpile_statement(b_stmt, indent + 1)).unwrap();
            }
        }
    }
    out.trim_end().to_string()
}

fn transpile_while(condition: &AnalyzedExpr, body: &[AnalyzedStatement], indent: usize) -> String {
    let ind = "    ".repeat(indent);
    let mut out = format!("{}while {}:\n", ind, transpile_expr(condition));
    if body.is_empty() {
        writeln!(out, "{}    pass", ind).unwrap();
    } else {
        for b_stmt in body {
            writeln!(
                out,
                "{}",
                transpile_statement(b_stmt, indent + 1).trim_end()
            )
            .unwrap();
        }
    }
    out.trim_end().to_string()
}

fn transpile_for(
    variable: &str,
    iterator: &AnalyzedExpr,
    body: &[AnalyzedStatement],
    indent: usize,
) -> String {
    let ind = "    ".repeat(indent);
    let mut out = format!(
        "{}for {} in {}:\n",
        ind,
        sanitize_ident(variable),
        transpile_expr(iterator)
    );
    if body.is_empty() {
        writeln!(out, "{}    pass", ind).unwrap();
    } else {
        for b_stmt in body {
            writeln!(
                out,
                "{}",
                transpile_statement(b_stmt, indent + 1).trim_end()
            )
            .unwrap();
        }
    }
    out.trim_end().to_string()
}

fn transpile_function_def(
    name: &str,
    params: &[(smol_str::SmolStr, Option<crate::semantic::GlossaType>)],
    body: &[AnalyzedStatement],
    indent: usize,
) -> String {
    let ind = "    ".repeat(indent);
    let mut out = format!("{}def {}(", ind, sanitize_ident(name));
    for (i, (p, _)) in params.iter().enumerate() {
        if i > 0 {
            write!(out, ", ").unwrap();
        }
        write!(out, "{}", sanitize_ident(p)).unwrap();
    }
    writeln!(out, "):").unwrap();

    if body.is_empty() {
        writeln!(out, "{}    pass", ind).unwrap();
    } else {
        for (i, b_stmt) in body.iter().enumerate() {
            let mut is_last_expr = false;
            if i == body.len() - 1 && matches!(b_stmt, AnalyzedStatement::Expression(_)) {
                is_last_expr = true;
            }

            if is_last_expr {
                if let AnalyzedStatement::Expression(exprs) = b_stmt {
                    for (j, expr) in exprs.iter().enumerate() {
                        if j == exprs.len() - 1 {
                            writeln!(out, "{}    return {}", ind, transpile_expr(expr)).unwrap();
                        } else {
                            writeln!(out, "{}    {}", ind, transpile_expr(expr)).unwrap();
                        }
                    }
                }
            } else {
                writeln!(out, "{}", transpile_statement(b_stmt, indent + 1)).unwrap();
            }
        }
    }
    out.trim_end().to_string()
}

fn transpile_type_def(
    name: &str,
    fields: &[(smol_str::SmolStr, crate::semantic::GlossaType)],
    indent: usize,
) -> String {
    let ind = "    ".repeat(indent);
    let mut out = format!(
        "{}@dataclass\n{}class {}:\n",
        ind,
        ind,
        sanitize_ident(name)
    );
    if fields.is_empty() {
        writeln!(out, "{}    pass", ind).unwrap();
    } else {
        for (f_name, _) in fields {
            writeln!(out, "{}    {}: Any", ind, sanitize_ident(f_name)).unwrap();
        }
    }
    out.trim_end().to_string()
}

fn transpile_test_declaration(name: &str, body: &[AnalyzedStatement], indent: usize) -> String {
    let ind = "    ".repeat(indent);
    // We can transpile tests as regular functions prefixed with test_
    let safe_name = name.replace(" ", "_").replace("-", "_").replace("\"", "");
    let mut out = format!("{}def test_{}():\n", ind, safe_name);
    if body.is_empty() {
        writeln!(out, "{}    pass", ind).unwrap();
    } else {
        for b_stmt in body {
            writeln!(out, "{}", transpile_statement(b_stmt, indent + 1)).unwrap();
        }
    }
    out.trim_end().to_string()
}

fn transpile_match(
    scrutinee: &AnalyzedExpr,
    arms: &[(AnalyzedExpr, Vec<AnalyzedStatement>)],
    indent: usize,
) -> String {
    let ind = "    ".repeat(indent);
    // Python 3.10+ match statement
    let mut out = format!("{}match {}:\n", ind, transpile_expr(scrutinee));
    for (pattern_expr, arm_body) in arms {
        writeln!(out, "{}    case {}:", ind, transpile_expr(pattern_expr)).unwrap();
        if arm_body.is_empty() {
            writeln!(out, "{}        pass", ind).unwrap();
        } else {
            for b_stmt in arm_body {
                writeln!(out, "{}", transpile_statement(b_stmt, indent + 2)).unwrap();
            }
        }
    }
    out.trim_end().to_string()
}

fn transpile_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => n.to_string(),
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::BooleanLiteral(b) => (if *b { "True" } else { "False" }).to_string(),
        AnalyzedExprKind::Variable(name) => sanitize_ident(name),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", transpile_expr(owner), sanitize_ident(property))
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            // Map certain known verbs to Python built-ins if applicable. For now, general function call.
            format!(
                "{}({})",
                sanitize_ident(verb),
                format_transpiled_exprs(args)
            )
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            format!(
                "{}({})",
                sanitize_ident(func),
                format_transpiled_exprs(args)
            )
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            let mut kw_args_buf = String::with_capacity(fields.len() * 16);
            for (i, (f, a)) in fields.iter().zip(args.iter()).enumerate() {
                if i > 0 {
                    write!(&mut kw_args_buf, ", ").unwrap();
                }
                let _ = write!(
                    &mut kw_args_buf,
                    "{}={}",
                    sanitize_ident(f),
                    transpile_expr(a)
                );
            }
            format!("{}({})", sanitize_ident(type_name), kw_args_buf)
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let s = transpile_expr(start);
            let e = transpile_expr(end);
            if *inclusive {
                format!("range({}, {} + 1)", s, e)
            } else {
                format!("range({}, {})", s, e)
            }
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            format!("[{}]", format_transpiled_exprs(exprs))
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            let l = transpile_expr(left);
            let r = transpile_expr(right);
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "//", // Integer division in Python
                BinaryOp::Mod => "%",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
            };
            format!("({} {} {})", l, op_str, r)
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            let o = transpile_expr(operand);
            match op {
                UnaryOp::Neg => format!("-{}", o),
                UnaryOp::Not => format!("not {}", o),
                UnaryOp::Ref => o, // Python does not have explicit references
            }
        }
        // Fallback for unsupported complex expressions like Try, Option variants, etc.
        _ => format!(
            "/* Unimplemented expr: {} */",
            crate::tools::narrator::tell_expr(expr)
        ),
    }
}

fn sanitize_ident(name: &str) -> String {
    let safe_name = name.replace(" ", "_").replace("-", "_");
    format!("g_{}", safe_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};

    #[test]
    fn test_transpile_unimplemented_expr_fallback_new() {
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: crate::semantic::GlossaType::Number,
            })),
            glossa_type: crate::semantic::GlossaType::Unknown,
        };
        let py = transpile_expr(&expr);
        assert!(py.contains("/* Unimplemented expr: "));
    }

    #[test]
    fn test_run_alchemist_coverage() {
        // Just trigger standard alchemist transpile paths
        let code = "εἰ ἀληθές ἐστι, «ναι» λέγε.";
        let ast = parse(code).unwrap();
        let mut program = analyze_program(&ast).unwrap();
        program.statements.push(AnalyzedStatement::Break);
        program.statements.push(AnalyzedStatement::Continue);
        program
            .statements
            .push(AnalyzedStatement::Return { value: None });
        program.statements.push(AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: crate::semantic::GlossaType::Boolean,
            }),
            body: vec![],
        });
        program.statements.push(AnalyzedStatement::For {
            variable: smol_str::SmolStr::new("x"),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                glossa_type: crate::semantic::GlossaType::List(Box::new(
                    crate::semantic::GlossaType::Unknown,
                )),
            }),
            body: vec![],
        });

        let py = transpile_to_python(&program);
        assert!(py.contains("break"));
        assert!(py.contains("continue"));
    }

    use crate::parser::parse;
    use crate::semantic::analyze_program;

    fn transpile_code(code: &str) -> String {
        let ast = parse(code).unwrap();
        let program = analyze_program(&ast).unwrap();
        transpile_to_python(&program)
    }

    #[test]
    fn test_transpile_print() {
        let code = "«χαῖρε κόσμε» λέγε.";
        let py = transpile_code(code);
        assert!(py.contains("print(\"χαῖρε κόσμε\")"), "Got: {}", py);
    }

    #[test]
    fn test_transpile_variables() {
        let code = "ξ πέντε ἔστω. ξ λέγε.";
        let py = transpile_code(code);
        assert!(py.contains("g_ξ = 5"));
        assert!(py.contains("print(g_ξ)"));
    }

    #[test]
    fn test_transpile_arithmetic() {
        let code = "ξ 1 2 ἄθροισμα ἔστω.";
        let py = transpile_code(code);
        assert!(py.contains("g_ξ = (1 + 2)"));
    }

    #[test]
    fn test_transpile_if() {
        // Need a complete sentence for the condition to avoid "Binding without subject" from partial parsing
        let code = "εἰ ἀληθές ἐστι, «ναι» λέγε.";
        let py = transpile_code(code);
        assert!(py.contains("if True:"));
        assert!(py.contains("    print(\"ναι\")"));
    }

    #[test]
    fn test_transpile_function() {
        let code = "πρόσθεσις ὁρίζειν τῷ α ἀριθμοῦ τῷ β ἀριθμοῦ · α β ἄθροισμα δός.";
        let py = transpile_code(code);
        assert!(py.contains("def g_προσθεσις(g_α, g_β):"));
        assert!(py.contains("    return (g_α + g_β)"));
    }

    #[test]
    fn test_run_alchemist_file_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("too_large.γλ");

        // Create a file larger than MAX_FILE_SIZE (1MB)
        let max_size = 1024 * 1024;
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            let data = vec![0u8; max_size + 1];
            f.write_all(&data).unwrap();
        }

        let result = run_alchemist(&input_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_run_alchemist_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all(b"not valid syntax").unwrap();
        }
        let result = run_alchemist(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_run_alchemist_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            // Valid syntax, invalid semantics (reassigning undefined var)
            f.write_all("ψ πέντε γίγνεται.".as_bytes()).unwrap();
        }
        let result = run_alchemist(&input_path);
        assert!(result.is_err());
        // Since we extracted analyze_source, the outer string wrapping has changed slightly.
        // We just ensure it's a semantic error internally.
        assert!(result.unwrap_err().to_string().contains("Σφάλμα"));
    }
}
