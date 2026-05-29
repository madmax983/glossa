//! The Herald (ὁ Κῆρυξ) - JSON AST Exporter
//!
//! This module implements "The Herald", a tool that exports the semantic Abstract Syntax Tree
//! (`AnalyzedProgram`) into a structured JSON representation.
//!
//! # Purpose
//!
//! The Herald allows external tools (IDEs, language servers, or other compilers)
//! to ingest the semantic structure of a ΓΛΩΣΣΑ program without needing to parse
//! Ancient Greek or interface directly with the Rust codebase. It acts as the
//! universal messenger for the compiler's insights.

use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedMethod, AnalyzedProgram, AnalyzedStatement,
    CaptureMode, GlossaType,
};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use miette::Result;
use std::path::Path;

pub fn run_herald(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Κῆρυξ (Exporting JSON)", "📣");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let json = serialize_program(&program);
    status.success();
    println!("{}", json);

    Ok(())
}

fn escape_json(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn serialize_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => r#"{"type":"Number"}"#.to_string(),
        GlossaType::String => r#"{"type":"String"}"#.to_string(),
        GlossaType::Boolean => r#"{"type":"Boolean"}"#.to_string(),
        GlossaType::List(inner) => {
            format!(r#"{{"type":"List","inner":{}}}"#, serialize_type(inner))
        }
        GlossaType::Set(inner) => format!(r#"{{"type":"Set","inner":{}}}"#, serialize_type(inner)),
        GlossaType::Map(k, v) => format!(
            r#"{{"type":"Map","key":{},"value":{}}}"#,
            serialize_type(k),
            serialize_type(v)
        ),
        GlossaType::Option(inner) => {
            format!(r#"{{"type":"Option","inner":{}}}"#, serialize_type(inner))
        }
        GlossaType::Result(ok, err) => format!(
            r#"{{"type":"Result","ok":{},"err":{}}}"#,
            serialize_type(ok),
            serialize_type(err)
        ),
        GlossaType::Struct { name, .. } => format!(
            r#"{{"type":"Struct","name":"{}"}}"#,
            escape_json(name.as_str())
        ),
        GlossaType::Function { params, returns } => {
            let params_json = params
                .iter()
                .map(serialize_type)
                .collect::<Vec<_>>()
                .join(",");
            let returns_json = serialize_type(returns);
            format!(
                r#"{{"type":"Function","params":[{}],"returns":{}}}"#,
                params_json, returns_json
            )
        }
        GlossaType::Unit => r#"{"type":"Unit"}"#.to_string(),
        GlossaType::Unknown => r#"{"type":"Unknown"}"#.to_string(),
    }
}

fn serialize_capture_mode(mode: &CaptureMode) -> &'static str {
    match mode {
        CaptureMode::Borrow => r#""Borrow""#,
        CaptureMode::Move => r#""Move""#,
    }
}

fn serialize_expr(expr: &AnalyzedExpr) -> String {
    let kind_json = stacker::maybe_grow(32 * 1024, 1024 * 1024, || match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            format!(r#"{{"type":"StringLiteral","value":"{}"}}"#, escape_json(s))
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            format!(r#"{{"type":"NumberLiteral","value":{}}}"#, n)
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            format!(r#"{{"type":"BooleanLiteral","value":{}}}"#, b)
        }
        AnalyzedExprKind::Variable(v) => format!(
            r#"{{"type":"Variable","name":"{}"}}"#,
            escape_json(v.as_str())
        ),
        AnalyzedExprKind::PropertyAccess { owner, property } => format!(
            r#"{{"type":"PropertyAccess","owner":{},"property":"{}"}}"#,
            serialize_expr(owner),
            escape_json(property.as_str())
        ),
        AnalyzedExprKind::VerbCall { verb, args } => format!(
            r#"{{"type":"VerbCall","verb":"{}","args":[{}]}}"#,
            escape_json(verb.as_str()),
            args.iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedExprKind::BinOp { left, op, right } => format!(
            r#"{{"type":"BinOp","left":{},"op":"{:?}","right":{}}}"#,
            serialize_expr(left),
            op,
            serialize_expr(right)
        ),
        AnalyzedExprKind::UnaryOp { op, operand } => format!(
            r#"{{"type":"UnaryOp","op":"{:?}","operand":{}}}"#,
            op,
            serialize_expr(operand)
        ),
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => format!(
            r#"{{"type":"Range","start":{},"end":{},"inclusive":{}}}"#,
            serialize_expr(start),
            serialize_expr(end),
            inclusive
        ),
        AnalyzedExprKind::ArrayLiteral(v) => format!(
            r#"{{"type":"ArrayLiteral","elements":[{}]}}"#,
            v.iter().map(serialize_expr).collect::<Vec<_>>().join(",")
        ),
        AnalyzedExprKind::Some(v) => format!(r#"{{"type":"Some","value":{}}}"#, serialize_expr(v)),
        AnalyzedExprKind::None => r#"{"type":"None"}"#.to_string(),
        AnalyzedExprKind::Ok(v) => format!(r#"{{"type":"Ok","value":{}}}"#, serialize_expr(v)),
        AnalyzedExprKind::Err(v) => format!(r#"{{"type":"Err","value":{}}}"#, serialize_expr(v)),
        AnalyzedExprKind::Unwrap(v) => {
            format!(r#"{{"type":"Unwrap","value":{}}}"#, serialize_expr(v))
        }
        AnalyzedExprKind::Try(v) => format!(r#"{{"type":"Try","value":{}}}"#, serialize_expr(v)),
        AnalyzedExprKind::IndexAccess { array, index } => format!(
            r#"{{"type":"IndexAccess","array":{},"index":{}}}"#,
            serialize_expr(array),
            serialize_expr(index)
        ),
        AnalyzedExprKind::FunctionCall { func, args } => format!(
            r#"{{"type":"FunctionCall","func":"{}","args":[{}]}}"#,
            escape_json(func.as_str()),
            args.iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => format!(
            r#"{{"type":"MethodCall","receiver":{},"method":"{}","args":[{}]}}"#,
            serialize_expr(receiver),
            escape_json(method.as_str()),
            args.iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => format!(
            r#"{{"type":"StructInstantiation","type_name":"{}","fields":[{}],"args":[{}]}}"#,
            escape_json(type_name.as_str()),
            fields
                .iter()
                .map(|f| format!(r#""{}""#, escape_json(f.as_str())))
                .collect::<Vec<_>>()
                .join(","),
            args.iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => format!(
            r#"{{"type":"Lambda","params":[{}],"body":{},"capture_mode":{}}}"#,
            params
                .iter()
                .map(|p| format!(r#""{}""#, escape_json(p.as_str())))
                .collect::<Vec<_>>()
                .join(","),
            serialize_expr(body),
            serialize_capture_mode(capture_mode)
        ),
        AnalyzedExprKind::CollectionNew { collection_type } => format!(
            r#"{{"type":"CollectionNew","collection_type":"{}"}}"#,
            escape_json(collection_type)
        ),
        AnalyzedExprKind::Assert { condition } => format!(
            r#"{{"type":"Assert","condition":{}}}"#,
            serialize_expr(condition)
        ),
        AnalyzedExprKind::AssertEq { left, right } => format!(
            r#"{{"type":"AssertEq","left":{},"right":{}}}"#,
            serialize_expr(left),
            serialize_expr(right)
        ),
    });

    format!(
        r#"{{"expr":{},"glossa_type":{}}}"#,
        kind_json,
        serialize_type(&expr.glossa_type)
    )
}

fn serialize_method(method: &AnalyzedMethod) -> String {
    let params_json = method
        .params
        .iter()
        .map(|(n, t)| {
            format!(
                r#"{{"name":"{}","type":{}}}"#,
                escape_json(n.as_str()),
                serialize_type(t)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let body_json = method
        .body
        .as_ref()
        .map(|b| {
            format!(
                r#"[{}]"#,
                b.iter()
                    .map(serialize_statement)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .unwrap_or_else(|| "null".to_string());
    let ret_json = method
        .return_type
        .as_ref()
        .map(serialize_type)
        .unwrap_or_else(|| "null".to_string());
    format!(
        r#"{{"name":"{}","params":[{}],"body":{},"return_type":{}}}"#,
        escape_json(method.name.as_str()),
        params_json,
        body_json,
        ret_json
    )
}

fn serialize_statement(stmt: &AnalyzedStatement) -> String {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => format!(
            r#"{{"type":"Binding","name":"{}","value":{},"mutable":{}}}"#,
            escape_json(name.as_str()),
            serialize_expr(value),
            mutable
        ),
        AnalyzedStatement::Assignment { name, value } => format!(
            r#"{{"type":"Assignment","name":"{}","value":{}}}"#,
            escape_json(name.as_str()),
            serialize_expr(value)
        ),
        AnalyzedStatement::Print(exprs) => format!(
            r#"{{"type":"Print","exprs":[{}]}}"#,
            exprs
                .iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedStatement::Expression(exprs) => format!(
            r#"{{"type":"Expression","exprs":[{}]}}"#,
            exprs
                .iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedStatement::Query(exprs) => format!(
            r#"{{"type":"Query","exprs":[{}]}}"#,
            exprs
                .iter()
                .map(serialize_expr)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            let then_json = then_body
                .iter()
                .map(serialize_statement)
                .collect::<Vec<_>>()
                .join(",");
            let else_json = else_body
                .as_ref()
                .map(|b| {
                    format!(
                        r#"[{}]"#,
                        b.iter()
                            .map(serialize_statement)
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            format!(
                r#"{{"type":"If","condition":{},"then_body":[{}],"else_body":{}}}"#,
                serialize_expr(condition),
                then_json,
                else_json
            )
        }
        AnalyzedStatement::While { condition, body } => format!(
            r#"{{"type":"While","condition":{},"body":[{}]}}"#,
            serialize_expr(condition),
            body.iter()
                .map(serialize_statement)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => format!(
            r#"{{"type":"For","variable":"{}","iterator":{},"body":[{}]}}"#,
            escape_json(variable.as_str()),
            serialize_expr(iterator),
            body.iter()
                .map(serialize_statement)
                .collect::<Vec<_>>()
                .join(",")
        ),
        AnalyzedStatement::Match { scrutinee, arms } => {
            let arms_json = arms
                .iter()
                .map(|(p, b)| {
                    format!(
                        r#"{{"pattern":{},"body":[{}]}}"#,
                        serialize_expr(p),
                        b.iter()
                            .map(serialize_statement)
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"Match","scrutinee":{},"arms":[{}]}}"#,
                serialize_expr(scrutinee),
                arms_json
            )
        }
        AnalyzedStatement::Break => r#"{"type":"Break"}"#.to_string(),
        AnalyzedStatement::Continue => r#"{"type":"Continue"}"#.to_string(),
        AnalyzedStatement::Return { value } => {
            let val_json = value
                .as_ref()
                .map(|v| serialize_expr(v))
                .unwrap_or_else(|| "null".to_string());
            format!(r#"{{"type":"Return","value":{}}}"#, val_json)
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            return_type,
            body,
        } => {
            let params_json = params
                .iter()
                .map(|(n, t)| {
                    format!(
                        r#"{{"name":"{}","type":{}}}"#,
                        escape_json(n.as_str()),
                        t.as_ref()
                            .map(serialize_type)
                            .unwrap_or_else(|| "null".to_string())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let ret_json = return_type
                .as_ref()
                .map(serialize_type)
                .unwrap_or_else(|| "null".to_string());
            let body_json = body
                .iter()
                .map(serialize_statement)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"FunctionDef","name":"{}","params":[{}],"return_type":{},"body":[{}]}}"#,
                escape_json(name.as_str()),
                params_json,
                ret_json,
                body_json
            )
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let fields_json = fields
                .iter()
                .map(|(n, t)| {
                    format!(
                        r#"{{"name":"{}","type":{}}}"#,
                        escape_json(n.as_str()),
                        serialize_type(t)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"TypeDefinition","name":"{}","fields":[{}]}}"#,
                escape_json(name.as_str()),
                fields_json
            )
        }
        AnalyzedStatement::TraitDefinition { name, methods } => {
            let methods_json = methods
                .iter()
                .map(serialize_method)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"TraitDefinition","name":"{}","methods":[{}]}}"#,
                escape_json(name.as_str()),
                methods_json
            )
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => {
            let methods_json = methods
                .iter()
                .map(serialize_method)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"TraitImplementation","trait_name":"{}","type_name":"{}","methods":[{}]}}"#,
                escape_json(trait_name.as_str()),
                escape_json(type_name.as_str()),
                methods_json
            )
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            let body_json = body
                .iter()
                .map(serialize_statement)
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"TestDeclaration","name":"{}","body":[{}]}}"#,
                escape_json(name.as_str()),
                body_json
            )
        }
    })
}

pub fn serialize_program(program: &AnalyzedProgram) -> String {
    let stmts_json = program
        .statements
        .iter()
        .map(serialize_statement)
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"statements":[{}]}}"#, stmts_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
    };
    use smol_str::SmolStr;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello \"world\""), "hello \\\"world\\\"");
    }

    #[test]
    fn test_serialize_program() {
        let program = AnalyzedProgram {
            statements: vec![AnalyzedStatement::Binding {
                name: SmolStr::new("x"),
                value: AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(42),
                    glossa_type: GlossaType::Number,
                },
                mutable: false,
            }],
            scope: Scope::new(),
        };
        let json = serialize_program(&program);
        assert_eq!(
            json,
            r#"{"statements":[{"type":"Binding","name":"x","value":{"expr":{"type":"NumberLiteral","value":42},"glossa_type":{"type":"Number"}},"mutable":false}]}"#
        );
    }
}
