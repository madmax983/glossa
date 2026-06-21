//! The Scribe (ὁ Γραμματεύς) - JSON AST Exporter
//!
//! This module exports the `AnalyzedProgram` to a structured JSON format.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

pub fn run_scribe(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Γραμματεύς (Exporting JSON AST)", "📜");

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

    let mut json_str = String::new();
    serialize_program(&program, &mut json_str);
    println!("{}", json_str);

    Ok(())
}

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn serialize_program(p: &AnalyzedProgram, s: &mut String) {
    let _ = write!(s, "{{\"statements\":[");
    for (i, stmt) in p.statements.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        serialize_statement(stmt, s);
    }
    let _ = write!(s, "]}}");
}

fn serialize_statement(stmt: &AnalyzedStatement, s: &mut String) {
    match stmt {
        AnalyzedStatement::Binding { name, value, mutable } => {
            let _ = write!(s, "{{\"type\":\"Binding\",\"name\":{},\"value\":", escape_json_string(name.as_str()));
            serialize_expr(value, s);
            let _ = write!(s, ",\"mutable\":{}}}", mutable);
        }
        AnalyzedStatement::Assignment { name, value } => {
            let _ = write!(s, "{{\"type\":\"Assignment\",\"name\":{},\"value\":", escape_json_string(name.as_str()));
            serialize_expr(value, s);
            s.push('}');
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let _ = write!(s, "{{\"type\":\"TypeDefinition\",\"name\":{},\"fields\":[", escape_json_string(name.as_str()));
            for (i, (n, t)) in fields.iter().enumerate() {
                if i > 0 { s.push(','); }
                let _ = write!(s, "{{\"name\":{},\"type_str\":{}}}", escape_json_string(n.as_str()), escape_json_string(&t.to_string()));
            }
            let _ = write!(s, "]}}");
        }
        _ => {
            let _ = write!(s, "{{\"type\":\"Unknown\"}}");
        }
    }
}

fn serialize_expr(expr: &AnalyzedExpr, s: &mut String) {
    let _ = write!(s, "{{\"glossa_type\":{},\"kind_data\":", escape_json_string(&expr.glossa_type.to_string()));
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(st) => {
            let _ = write!(s, "{{\"kind\":\"StringLiteral\",\"value\":{}}}", escape_json_string(st));
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            let _ = write!(s, "{{\"kind\":\"NumberLiteral\",\"value\":{}}}", n);
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            let _ = write!(s, "{{\"kind\":\"BooleanLiteral\",\"value\":{}}}", b);
        }
        AnalyzedExprKind::Variable(v) => {
            let _ = write!(s, "{{\"kind\":\"Variable\",\"name\":{}}}", escape_json_string(v.as_str()));
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            let _ = write!(s, "{{\"kind\":\"PropertyAccess\",\"owner\":");
            serialize_expr(owner, s);
            let _ = write!(s, ",\"property\":{}}}", escape_json_string(property.as_str()));
        }
        _ => {
            let _ = write!(s, "{{\"kind\":\"Unknown\"}}");
        }
    }
    s.push('}');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "\"hello\"");
        assert_eq!(escape_json_string("hello \"world\""), "\"hello \\\"world\\\"\"");
        assert_eq!(escape_json_string("hello\nworld"), "\"hello\\nworld\"");
    }
}
