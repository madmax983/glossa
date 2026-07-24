//! The Artisan (ὁ Τεχνίτης) - JSON Exporter
//!
//! This module implements the "Artisan" tool, which exports the
//! semantic Abstract Syntax Tree (AST) into a structured JSON format.
//!
//! # Purpose
//!
//! By exporting the `AnalyzedProgram` to JSON, we open the door for
//! interoperability with external IDEs, web dashboards, and tools
//! written in other languages.

use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType,
};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Runs the Artisan tool to generate a JSON representation of the program.
pub fn run_artisan(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Τεχνίτης (Exporting to JSON)", "🔨");

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
    println!("   {}", "Γ Λ Ω Σ Σ Α   A R T I S A N".bold().cyan());
    println!("   {}", "JSON AST Export".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("JSON Output")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    let formatted_code = format!("```json\n{}\n```", json_output.trim());
    table.add_row(vec![Cell::new(formatted_code)]);

    println!("{table}");
    println!();

    Ok(())
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn type_to_json(t: &GlossaType) -> String {
    match t {
        GlossaType::Number => "\"Number\"".to_string(),
        GlossaType::String => "\"String\"".to_string(),
        GlossaType::Boolean => "\"Boolean\"".to_string(),
        GlossaType::Unknown => "\"Unknown\"".to_string(),
        _ => "\"Complex\"".to_string(), // Simplified for the artisan MVP
    }
}

fn expr_to_json(expr: &AnalyzedExpr, indent: usize) -> String {
    let ind = "  ".repeat(indent);
    let inner_ind = "  ".repeat(indent + 1);

    let kind_json = match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => format!(
            "\"type\": \"NumberLiteral\",\n{}\"value\": {}",
            inner_ind, n
        ),
        AnalyzedExprKind::StringLiteral(s) => format!(
            "\"type\": \"StringLiteral\",\n{}\"value\": \"{}\"",
            inner_ind,
            escape_string(s)
        ),
        AnalyzedExprKind::BooleanLiteral(b) => format!(
            "\"type\": \"BooleanLiteral\",\n{}\"value\": {}",
            inner_ind, b
        ),
        AnalyzedExprKind::Variable(v) => format!(
            "\"type\": \"Variable\",\n{}\"name\": \"{}\"",
            inner_ind,
            escape_string(v.as_str())
        ),
        // Catch all for others to keep MVP simple but valid
        _ => "\"type\": \"Other\"".to_string(),
    };

    format!(
        "{{\n{}\"glossa_type\": {},\n{}{}\n{}}}",
        inner_ind,
        type_to_json(&expr.glossa_type),
        inner_ind,
        kind_json,
        ind
    )
}

fn statement_to_json(stmt: &AnalyzedStatement, indent: usize) -> String {
    let ind = "  ".repeat(indent);
    let inner_ind = "  ".repeat(indent + 1);

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            format!(
                "{{\n{}\"type\": \"Binding\",\n{}\"name\": \"{}\",\n{}\"mutable\": {},\n{}\"value\": {}\n{}}}",
                inner_ind,
                inner_ind,
                escape_string(name.as_str()),
                inner_ind,
                mutable,
                inner_ind,
                expr_to_json(value, indent + 1),
                ind
            )
        }
        AnalyzedStatement::Print(exprs) => {
            let exprs_json: Vec<String> =
                exprs.iter().map(|e| expr_to_json(e, indent + 2)).collect();
            format!(
                "{{\n{}\"type\": \"Print\",\n{}\"expressions\": [\n{}\n{}]\n{}}}",
                inner_ind,
                inner_ind,
                exprs_json.join(",\n"),
                inner_ind,
                ind
            )
        }
        AnalyzedStatement::Expression(exprs) => {
            let exprs_json: Vec<String> =
                exprs.iter().map(|e| expr_to_json(e, indent + 2)).collect();
            format!(
                "{{\n{}\"type\": \"Expression\",\n{}\"expressions\": [\n{}\n{}]\n{}}}",
                inner_ind,
                inner_ind,
                exprs_json.join(",\n"),
                inner_ind,
                ind
            )
        }
        AnalyzedStatement::Query(exprs) => {
            let exprs_json: Vec<String> =
                exprs.iter().map(|e| expr_to_json(e, indent + 2)).collect();
            format!(
                "{{\n{}\"type\": \"Query\",\n{}\"expressions\": [\n{}\n{}]\n{}}}",
                inner_ind,
                inner_ind,
                exprs_json.join(",\n"),
                inner_ind,
                ind
            )
        }
        AnalyzedStatement::Assignment { name, value } => {
            format!(
                "{{\n{}\"type\": \"Assignment\",\n{}\"name\": \"{}\",\n{}\"value\": {}\n{}}}",
                inner_ind,
                inner_ind,
                escape_string(name.as_str()),
                inner_ind,
                expr_to_json(value, indent + 1),
                ind
            )
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let fields_json: Vec<String> = fields
                .iter()
                .map(|(n, t)| {
                    format!(
                        "{{\"name\": \"{}\", \"type\": {}}}",
                        escape_string(n.as_str()),
                        type_to_json(t)
                    )
                })
                .collect();
            format!(
                "{{\n{}\"type\": \"TypeDefinition\",\n{}\"name\": \"{}\",\n{}\"fields\": [{}]\n{}}}",
                inner_ind,
                inner_ind,
                escape_string(name.as_str()),
                inner_ind,
                fields_json.join(", "),
                ind
            )
        }
        AnalyzedStatement::Return { value } => {
            let val_json = match value {
                Some(v) => expr_to_json(v, indent + 1),
                None => "null".to_string(),
            };
            format!(
                "{{\n{}\"type\": \"Return\",\n{}\"value\": {}\n{}}}",
                inner_ind, inner_ind, val_json, ind
            )
        }
        AnalyzedStatement::Break => {
            format!("{{\n{}\"type\": \"Break\"\n{}}}", inner_ind, ind)
        }
        AnalyzedStatement::Continue => {
            format!("{{\n{}\"type\": \"Continue\"\n{}}}", inner_ind, ind)
        }
        _ => {
            format!(
                "{{\n{}\"type\": \"UnsupportedStatement\"\n{}}}",
                inner_ind, ind
            )
        }
    }
}

pub fn program_to_json(program: &AnalyzedProgram) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"statements\": [\n");

    let stmts: Vec<String> = program
        .statements
        .iter()
        .map(|s| {
            let json = statement_to_json(s, 2);
            format!("    {}", json.trim_start())
        })
        .collect();

    out.push_str(&stmts.join(",\n"));
    out.push_str("\n  ]\n}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::Scope;

    #[test]
    fn test_artisan_json_export() {
        let mut program = AnalyzedProgram {
            statements: vec![],
            scope: Scope::new(),
        };

        program.statements.push(AnalyzedStatement::Binding {
            name: "x".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(42),
                glossa_type: GlossaType::Number,
            },
            mutable: false,
        });

        let json = program_to_json(&program);
        assert!(json.contains("\"type\": \"Binding\""));
        assert!(json.contains("\"name\": \"x\""));
        assert!(json.contains("\"value\": 42"));
    }
}
