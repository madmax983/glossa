//! The Astrolabe (ὁ Ἀστρολάβος) - JSON AST Exporter
//!
//! This module implements the "Astrolabe" tool, which parses a ΓΛΩΣΣΑ program
//! and exports its semantic AST (`AnalyzedProgram`) directly to a structured JSON file.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType};
use crate::tools::runner::analyze_source;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// A builder for constructing JSON strings manually without heavy dependencies.
struct JsonBuilder {
    buffer: String,
    indent: usize,
}

impl JsonBuilder {
    fn new() -> Self {
        Self {
            buffer: String::with_capacity(8192),
            indent: 0,
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..(self.indent * 2) {
            self.buffer.push(' ');
        }
    }

    fn begin_object(&mut self) {
        self.buffer.push_str("{\n");
        self.indent += 1;
    }

    fn end_object(&mut self, is_last: bool) {
        self.indent -= 1;
        self.push_indent();
        if is_last {
            self.buffer.push_str("}\n");
        } else {
            self.buffer.push_str("},\n");
        }
    }

    fn begin_array(&mut self) {
        self.buffer.push_str("[\n");
        self.indent += 1;
    }

    fn end_array(&mut self, is_last: bool) {
        self.indent -= 1;
        self.push_indent();
        if is_last {
            self.buffer.push_str("]\n");
        } else {
            self.buffer.push_str("],\n");
        }
    }

    fn write_key(&mut self, key: &str) {
        self.push_indent();
        write!(self.buffer, "\"{}\": ", key).unwrap();
    }

    fn write_string_field(&mut self, key: &str, value: &str, is_last: bool) {
        self.write_key(key);
        // Basic escaping for JSON strings
        let escaped = value
            .replace('\\', "\\\\")
            .replace('\"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        if is_last {
            writeln!(self.buffer, "\"{}\"", escaped).unwrap();
        } else {
            writeln!(self.buffer, "\"{}\",", escaped).unwrap();
        }
    }

    fn write_bool_field(&mut self, key: &str, value: bool, is_last: bool) {
        self.write_key(key);
        if is_last {
            writeln!(self.buffer, "{}", if value { "true" } else { "false" }).unwrap();
        } else {
            writeln!(self.buffer, "{},", if value { "true" } else { "false" }).unwrap();
        }
    }

    fn write_number_field(&mut self, key: &str, value: i64, is_last: bool) {
        self.write_key(key);
        if is_last {
            writeln!(self.buffer, "{}", value).unwrap();
        } else {
            writeln!(self.buffer, "{},", value).unwrap();
        }
    }

    fn build(self) -> String {
        self.buffer
    }
}

fn type_to_string(t: &GlossaType) -> String {
    t.to_string()
}

fn emit_expr(builder: &mut JsonBuilder, expr: &AnalyzedExpr, is_last: bool) {
    builder.begin_object();
    builder.write_string_field("type", &type_to_string(&expr.glossa_type), false);
    builder.write_key("kind");

    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            builder.begin_object();
            builder.write_string_field("variant", "StringLiteral", false);
            builder.write_string_field("value", s, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            builder.begin_object();
            builder.write_string_field("variant", "NumberLiteral", false);
            builder.write_number_field("value", *n, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            builder.begin_object();
            builder.write_string_field("variant", "BooleanLiteral", false);
            builder.write_bool_field("value", *b, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::Variable(v) => {
            builder.begin_object();
            builder.write_string_field("variant", "Variable", false);
            builder.write_string_field("name", v, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            builder.begin_object();
            builder.write_string_field("variant", "PropertyAccess", false);
            builder.write_string_field("property", property, false);
            builder.write_key("owner");
            emit_expr(builder, owner, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            builder.begin_object();
            builder.write_string_field("variant", "BinOp", false);
            builder.write_string_field("op", &format!("{:?}", op), false);
            builder.write_key("left");
            emit_expr(builder, left, false);
            builder.write_key("right");
            emit_expr(builder, right, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            builder.begin_object();
            builder.write_string_field("variant", "IndexAccess", false);
            builder.write_key("array");
            emit_expr(builder, array, false);
            builder.write_key("index");
            emit_expr(builder, index, true);
            builder.end_object(true);
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            builder.begin_object();
            builder.write_string_field("variant", "FunctionCall", false);
            builder.write_string_field("func", func, false);
            builder.write_key("args");
            builder.begin_array();
            for (i, arg) in args.iter().enumerate() {
                emit_expr(builder, arg, i == args.len() - 1);
            }
            builder.end_array(true);
            builder.end_object(true);
        }
        _ => {
            builder.begin_object();
            builder.write_string_field("variant", "Unknown", true);
            builder.end_object(true);
        }
    }

    builder.end_object(is_last);
}

fn emit_statement(builder: &mut JsonBuilder, stmt: &AnalyzedStatement, is_last: bool) {
    builder.begin_object();

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            builder.write_string_field("type", "Binding", false);
            builder.write_string_field("name", name, false);
            builder.write_bool_field("mutable", *mutable, false);
            builder.write_key("value");
            emit_expr(builder, value, true);
        }
        AnalyzedStatement::Assignment { name, value } => {
            builder.write_string_field("type", "Assignment", false);
            builder.write_string_field("name", name, false);
            builder.write_key("value");
            emit_expr(builder, value, true);
        }
        AnalyzedStatement::Print(exprs) => {
            builder.write_string_field("type", "Print", false);
            builder.write_key("exprs");
            builder.begin_array();
            for (i, expr) in exprs.iter().enumerate() {
                emit_expr(builder, expr, i == exprs.len() - 1);
            }
            builder.end_array(true);
        }
        AnalyzedStatement::Expression(exprs) => {
            builder.write_string_field("type", "Expression", false);
            builder.write_key("exprs");
            builder.begin_array();
            for (i, expr) in exprs.iter().enumerate() {
                emit_expr(builder, expr, i == exprs.len() - 1);
            }
            builder.end_array(true);
        }
        AnalyzedStatement::Query(exprs) => {
            builder.write_string_field("type", "Query", false);
            builder.write_key("exprs");
            builder.begin_array();
            for (i, expr) in exprs.iter().enumerate() {
                emit_expr(builder, expr, i == exprs.len() - 1);
            }
            builder.end_array(true);
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            builder.write_string_field("type", "If", false);
            builder.write_key("condition");
            emit_expr(builder, condition, false);
            builder.write_key("then_body");
            builder.begin_array();
            for (i, s) in then_body.iter().enumerate() {
                emit_statement(builder, s, i == then_body.len() - 1);
            }
            builder.end_array(else_body.is_none());
            if let Some(else_stmts) = else_body {
                builder.write_key("else_body");
                builder.begin_array();
                for (i, s) in else_stmts.iter().enumerate() {
                    emit_statement(builder, s, i == else_stmts.len() - 1);
                }
                builder.end_array(true);
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            builder.write_string_field("type", "TypeDefinition", false);
            builder.write_string_field("name", name, false);
            builder.write_key("fields");
            builder.begin_array();
            for (i, (fname, ftype)) in fields.iter().enumerate() {
                builder.begin_object();
                builder.write_string_field("name", fname, false);
                builder.write_string_field("type", &type_to_string(ftype), true);
                builder.end_object(i == fields.len() - 1);
            }
            builder.end_array(true);
        }
        _ => {
            builder.write_string_field("type", "Unknown", true);
        }
    }

    builder.end_object(is_last);
}

/// Runs the Astrolabe tool to export the AST to a JSON file.
pub fn run_astrolabe(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Ἀστρολάβος (Exporting AST)", "🧭");

    let program = match analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let mut builder = JsonBuilder::new();
    builder.begin_object();
    builder.write_key("statements");
    builder.begin_array();

    for (i, stmt) in program.statements.iter().enumerate() {
        emit_statement(&mut builder, stmt, i == program.statements.len() - 1);
    }

    builder.end_array(true);
    builder.end_object(true);

    let json_output = builder.build();
    let output_path = input.with_extension("ast.json");

    if let Err(e) = std::fs::write(&output_path, &json_output) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write JSON file: {}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A S T R O L A B E".bold().cyan());
    println!("   {}", "Semantic AST Exported".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_astrolabe_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.γλ");
        fs::write(&input_path, "ξ 5 ἔστω.\n").unwrap();

        let result = run_astrolabe(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("ast.json");
        assert!(output_path.exists());

        let json_content = fs::read_to_string(&output_path).unwrap();
        assert!(json_content.contains("\"type\": \"Binding\""));
        assert!(json_content.contains("\"name\": \"ξ\""));
        assert!(json_content.contains("\"variant\": \"NumberLiteral\""));
    }
}
