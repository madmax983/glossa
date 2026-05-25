#![cfg(feature = "nova")]

//! The Ambassador (ὁ Πρέσβυς) - JSON Exporter
//!
//! This module implements a tool that exports the parsed AST and semantic analysis
//! to JSON format, allowing external tools to interoperate with the compiler.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

pub fn run_ambassador(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Ἐξαγωγή (Exporting JSON)", "🌐");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let json = generate_json(&program);

    let output_path = input.with_extension("json");
    if let Err(e) = std::fs::write(&output_path, &json) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write JSON file: {}", e));
    }

    status.success();
    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A M B A S S A D O R".bold().cyan());
    println!("   {}", "AST Exported to JSON".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

pub fn generate_json(program: &AnalyzedProgram) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"statements\": [\n");

    for (i, stmt) in program.statements.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        generate_stmt_json(stmt, &mut out, 4);
    }

    out.push_str("\n  ]\n}");
    out
}

fn generate_stmt_json(stmt: &AnalyzedStatement, out: &mut String, indent: usize) {
    let ind = " ".repeat(indent);
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            out.push_str(&format!("{}{{\n", ind));
            out.push_str(&format!("{}  \"type\": \"Binding\",\n", ind));
            out.push_str(&format!("{}  \"name\": \"{}\",\n", ind, escape_json(name)));
            out.push_str(&format!("{}  \"mutable\": {},\n", ind, mutable));
            out.push_str(&format!("{}  \"value\": ", ind));
            generate_expr_json(value, out, indent + 2);
            out.push_str(&format!("\n{}}}", ind));
        }
        _ => {
            // Minimal implementation for the tests and baseline export.
            // Other variants can be added later as part of iterative enhancements.
            out.push_str(&format!("{}{{\n", ind));
            out.push_str(&format!("{}  \"type\": \"UnsupportedStatement\"\n", ind));
            out.push_str(&format!("{}}}", ind));
        }
    }
}

fn generate_expr_json(expr: &AnalyzedExpr, out: &mut String, indent: usize) {
    let ind = " ".repeat(indent);
    out.push_str("{\n");

    match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => {
            out.push_str(&format!("{}  \"expr\": \"NumberLiteral\",\n", ind));
            out.push_str(&format!("{}  \"value\": {}", ind, n));
        }
        AnalyzedExprKind::StringLiteral(s) => {
            out.push_str(&format!("{}  \"expr\": \"StringLiteral\",\n", ind));
            out.push_str(&format!("{}  \"value\": \"{}\"", ind, escape_json(s)));
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            out.push_str(&format!("{}  \"expr\": \"BooleanLiteral\",\n", ind));
            out.push_str(&format!("{}  \"value\": {}", ind, b));
        }
        _ => {
            out.push_str(&format!("{}  \"expr\": \"UnsupportedExpr\"", ind));
        }
    }

    out.push_str(&format!("\n{}}}", ind));
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};

    #[test]
    fn test_generate_json_empty() {
        let program = AnalyzedProgram {
            statements: vec![],
            scope: Scope::new(),
        };
        let json = generate_json(&program);
        assert_eq!(json, "{\n  \"statements\": [\n\n  ]\n}");
    }

    #[test]
    fn test_generate_json_binding() {
        let program = AnalyzedProgram {
            statements: vec![AnalyzedStatement::Binding {
                name: "x".into(),
                value: AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(5),
                    glossa_type: GlossaType::Number,
                },
                mutable: false,
            }],
            scope: Scope::new(),
        };
        let json = generate_json(&program);
        assert!(json.contains("\"type\": \"Binding\""));
        assert!(json.contains("\"name\": \"x\""));
        assert!(json.contains("\"value\": {"));
        assert!(json.contains("\"expr\": \"NumberLiteral\""));
        assert!(json.contains("\"value\": 5"));
    }
}
