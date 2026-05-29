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

    #[test]
    fn test_serialize_types() {
        assert_eq!(serialize_type(&GlossaType::Number), r#"{"type":"Number"}"#);
        assert_eq!(serialize_type(&GlossaType::String), r#"{"type":"String"}"#);
        assert_eq!(serialize_type(&GlossaType::Boolean), r#"{"type":"Boolean"}"#);
        assert_eq!(serialize_type(&GlossaType::Unit), r#"{"type":"Unit"}"#);
        assert_eq!(serialize_type(&GlossaType::Unknown), r#"{"type":"Unknown"}"#);
        assert_eq!(serialize_type(&GlossaType::List(Box::new(GlossaType::Number))), r#"{"type":"List","inner":{"type":"Number"}}"#);
        assert_eq!(serialize_type(&GlossaType::Set(Box::new(GlossaType::Number))), r#"{"type":"Set","inner":{"type":"Number"}}"#);
        assert_eq!(serialize_type(&GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number))), r#"{"type":"Map","key":{"type":"String"},"value":{"type":"Number"}}"#);
        assert_eq!(serialize_type(&GlossaType::Option(Box::new(GlossaType::Number))), r#"{"type":"Option","inner":{"type":"Number"}}"#);
        assert_eq!(serialize_type(&GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String))), r#"{"type":"Result","ok":{"type":"Number"},"err":{"type":"String"}}"#);
        assert_eq!(serialize_type(&GlossaType::Struct { name: SmolStr::new("User"), gender: crate::morphology::Gender::Masculine, fields: vec![] }), r#"{"type":"Struct","name":"User"}"#);
        assert_eq!(serialize_type(&GlossaType::Function { params: vec![GlossaType::Number], returns: Box::new(GlossaType::Boolean) }), r#"{"type":"Function","params":[{"type":"Number"}],"returns":{"type":"Boolean"}}"#);
    }

    #[test]
    fn test_serialize_exprs() {
        let dummy_type = GlossaType::Number;

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("test".to_string()), glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"StringLiteral","value":"test"},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(true), glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"BooleanLiteral","value":true},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Variable(SmolStr::new("x")), glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"Variable","name":"x"},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::PropertyAccess { owner: Box::new(expr.clone()), property: SmolStr::new("y") }, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"PropertyAccess","owner":{"expr":{"type":"Variable","name":"x"},"glossa_type":{"type":"Number"}},"property":"y"},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::VerbCall { verb: SmolStr::new("run"), args: vec![] }, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"VerbCall","verb":"run","args":[]},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::BinOp { left: Box::new(expr.clone()), op: crate::morphology::lexicon::BinaryOp::Add, right: Box::new(expr.clone()) }, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"BinOp","left":{"expr":{"type":"VerbCall","verb":"run","args":[]},"glossa_type":{"type":"Number"}},"op":"Add","right":{"expr":{"type":"VerbCall","verb":"run","args":[]},"glossa_type":{"type":"Number"}}},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::UnaryOp { op: crate::morphology::lexicon::UnaryOp::Not, operand: Box::new(expr.clone()) }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"UnaryOp","op":"Not""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Range { start: Box::new(expr.clone()), end: Box::new(expr.clone()), inclusive: true }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Range""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::ArrayLiteral(vec![]), glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"ArrayLiteral","elements":[]},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Some(Box::new(expr.clone())), glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Some""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"None"},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Ok(Box::new(expr.clone())), glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Ok""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Err(Box::new(expr.clone())), glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Err""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Unwrap(Box::new(expr.clone())), glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Unwrap""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Try(Box::new(expr.clone())), glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Try""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::IndexAccess { array: Box::new(expr.clone()), index: Box::new(expr.clone()) }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"IndexAccess""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::FunctionCall { func: SmolStr::new("f"), args: vec![] }, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"FunctionCall","func":"f","args":[]},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::MethodCall { receiver: Box::new(expr.clone()), method: SmolStr::new("m"), args: vec![] }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"MethodCall""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::StructInstantiation { type_name: SmolStr::new("User"), fields: vec![], args: vec![] }, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"StructInstantiation","type_name":"User","fields":[],"args":[]},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Lambda { params: vec![], body: Box::new(expr.clone()), capture_mode: CaptureMode::Borrow }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Lambda""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::CollectionNew { collection_type: "List".to_string() }, glossa_type: dummy_type.clone() };
        assert_eq!(serialize_expr(&expr), r#"{"expr":{"type":"CollectionNew","collection_type":"List"},"glossa_type":{"type":"Number"}}"#);

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::Assert { condition: Box::new(expr.clone()) }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"Assert""#));

        let expr = AnalyzedExpr { expr: AnalyzedExprKind::AssertEq { left: Box::new(expr.clone()), right: Box::new(expr.clone()) }, glossa_type: dummy_type.clone() };
        assert!(serialize_expr(&expr).contains(r#"{"type":"AssertEq""#));
    }

    #[test]
    fn test_serialize_statements() {
        let dummy_expr = AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unknown };

        let stmt = AnalyzedStatement::Assignment { name: SmolStr::new("x"), value: dummy_expr.clone() };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"Assignment""#));

        let stmt = AnalyzedStatement::Print(vec![]);
        assert_eq!(serialize_statement(&stmt), r#"{"type":"Print","exprs":[]}"#);

        let stmt = AnalyzedStatement::Expression(vec![]);
        assert_eq!(serialize_statement(&stmt), r#"{"type":"Expression","exprs":[]}"#);

        let stmt = AnalyzedStatement::Query(vec![]);
        assert_eq!(serialize_statement(&stmt), r#"{"type":"Query","exprs":[]}"#);

        let stmt = AnalyzedStatement::If { condition: Box::new(dummy_expr.clone()), then_body: vec![], else_body: None };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"If""#));

        let stmt = AnalyzedStatement::If { condition: Box::new(dummy_expr.clone()), then_body: vec![], else_body: Some(vec![]) };
        assert!(serialize_statement(&stmt).contains(r#"else_body":[]"#));

        let stmt = AnalyzedStatement::While { condition: Box::new(dummy_expr.clone()), body: vec![] };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"While""#));

        let stmt = AnalyzedStatement::For { variable: SmolStr::new("x"), iterator: dummy_expr.clone(), body: vec![] };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"For""#));

        let stmt = AnalyzedStatement::Match { scrutinee: dummy_expr.clone(), arms: vec![(dummy_expr.clone(), vec![])] };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"Match""#));

        let stmt = AnalyzedStatement::Break;
        assert_eq!(serialize_statement(&stmt), r#"{"type":"Break"}"#);

        let stmt = AnalyzedStatement::Continue;
        assert_eq!(serialize_statement(&stmt), r#"{"type":"Continue"}"#);

        let stmt = AnalyzedStatement::Return { value: None };
        assert_eq!(serialize_statement(&stmt), r#"{"type":"Return","value":null}"#);

        let stmt = AnalyzedStatement::Return { value: Some(dummy_expr.clone()) };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"Return""#));

        let stmt = AnalyzedStatement::FunctionDef { name: SmolStr::new("f"), params: vec![], return_type: None, body: vec![] };
        assert_eq!(serialize_statement(&stmt), r#"{"type":"FunctionDef","name":"f","params":[],"return_type":null,"body":[]}"#);

        let stmt = AnalyzedStatement::FunctionDef { name: SmolStr::new("f"), params: vec![(SmolStr::new("x"), Some(GlossaType::Number))], return_type: Some(GlossaType::Unit), body: vec![] };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"FunctionDef""#));

        let stmt = AnalyzedStatement::TypeDefinition { name: SmolStr::new("User"), fields: vec![(SmolStr::new("x"), GlossaType::Number)] };
        assert_eq!(serialize_statement(&stmt), r#"{"type":"TypeDefinition","name":"User","fields":[{"name":"x","type":{"type":"Number"}}]}"#);

        let method = AnalyzedMethod { name: SmolStr::new("m"), params: vec![], body: None, return_type: None };
        let method2 = AnalyzedMethod { name: SmolStr::new("m2"), params: vec![(SmolStr::new("x"), GlossaType::Number)], body: Some(vec![]), return_type: Some(GlossaType::Unit) };
        let stmt = AnalyzedStatement::TraitDefinition { name: SmolStr::new("Tr"), methods: vec![method, method2] };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"TraitDefinition""#));

        let method = AnalyzedMethod { name: SmolStr::new("m"), params: vec![], body: None, return_type: None };
        let stmt = AnalyzedStatement::TraitImplementation { trait_name: SmolStr::new("Tr"), type_name: SmolStr::new("User"), methods: vec![method] };
        assert!(serialize_statement(&stmt).contains(r#"{"type":"TraitImplementation""#));

        let stmt = AnalyzedStatement::TestDeclaration { name: SmolStr::new("t"), body: vec![] };
        assert_eq!(serialize_statement(&stmt), r#"{"type":"TestDeclaration","name":"t","body":[]}"#);
    }

}
