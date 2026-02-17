//! The Narrator Tool ("Bard")
//!
//! This module implements the "Bard" functionality, which translates the semantic meaning
//! of a ΓΛΩΣΣΑ program into a readable English narrative, known as "The Scroll of Logic".
//!
//! # Purpose
//!
//! This tool serves two main purposes:
//! 1. **Debugging**: It allows developers to verify how the compiler is interpreting their code.
//!    If the English narrative doesn't match their intent, there's likely a parsing or semantic error.
//! 2. **Education**: It helps users understand the mapping between Ancient Greek syntax and
//!    computational logic.
//!
//! # How it works
//!
//! The `tell_tale` function takes an [`AnalyzedProgram`] (the output of the semantic analysis phase)
//! and recursively traverses the AST, generating English sentences for each statement and expression.
//!
//! # Example
//!
//! ```glossa
//! ξ πέντε ἔστω.
//! ```
//!
//! Becomes:
//!
//! > Let there be a variable named `ξ` with the value the number 5.

use crate::semantic::CaptureMode;
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType,
};

/// Tells the tale of the program in English.
///
/// This function translates the semantic meaning of the program into a readable English narrative.
/// It acts as the entry point for the "Bard" tool.
///
/// # Examples
///
/// ```
/// use glossa::tools::narrator::tell_tale;
/// use glossa::parser::parse;
/// use glossa::semantic::analyze_program;
///
/// let source = "ξ πέντε ἔστω.";
/// let ast = parse(source).unwrap();
/// let analyzed = analyze_program(&ast).unwrap();
///
/// let tale = tell_tale(&analyzed);
/// assert!(tale.contains("Let there be a variable named `ξ`"));
/// ```
pub fn tell_tale(program: &AnalyzedProgram) -> String {
    let mut tale = String::new();
    tale.push_str("The Scroll of Logic begins...\n\n");

    for stmt in &program.statements {
        tale.push_str(&tell_statement(stmt, 0));
        tale.push('\n');
    }

    tale.push_str("\n...and thus the ritual is complete.");
    tale
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn tell_statement(stmt: &AnalyzedStatement, level: usize) -> String {
    let prefix = indent(level);
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let mutability = if *mutable { "mutable " } else { "" };
            format!(
                "{}Let there be a {}variable named `{}` with the value {}.",
                prefix,
                mutability,
                name,
                tell_expr(value)
            )
        }
        AnalyzedStatement::Assignment { name, value } => {
            format!(
                "{}Update `{}` to become {}.",
                prefix,
                name,
                tell_expr(value)
            )
        }
        AnalyzedStatement::Print(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            format!("{}Proclaim to the world: {}.", prefix, expr_strs.join(", "))
        }
        AnalyzedStatement::Expression(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            format!("{}Perform the following: {}.", prefix, expr_strs.join(", "))
        }
        AnalyzedStatement::Query(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            format!(
                "{}Query the oracle about: {}.",
                prefix,
                expr_strs.join(", ")
            )
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut s = format!(
                "{}If it is true that {}, then:\n",
                prefix,
                tell_expr(condition)
            );
            for stmt in then_body {
                s.push_str(&tell_statement(stmt, level + 1));
                s.push('\n');
            }
            if let Some(else_stmts) = else_body {
                s.push_str(&format!("{}Otherwise:\n", prefix));
                for stmt in else_stmts {
                    s.push_str(&tell_statement(stmt, level + 1));
                    s.push('\n');
                }
            }
            s
        }
        AnalyzedStatement::While { condition, body } => {
            let mut s = format!("{}As long as {}, repeat:\n", prefix, tell_expr(condition));
            for stmt in body {
                s.push_str(&tell_statement(stmt, level + 1));
                s.push('\n');
            }
            s
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            let mut s = format!(
                "{}For each `{}` found in {}, do:\n",
                prefix,
                variable,
                tell_expr(iterator)
            );
            for stmt in body {
                s.push_str(&tell_statement(stmt, level + 1));
                s.push('\n');
            }
            s
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            let mut s = format!(
                "{}Consider the nature of {}:\n",
                prefix,
                tell_expr(scrutinee)
            );
            for (pat, body) in arms {
                s.push_str(&format!("{}  In the case of {}:\n", prefix, tell_expr(pat)));
                for stmt in body {
                    s.push_str(&tell_statement(stmt, level + 2));
                    s.push('\n');
                }
            }
            s
        }
        AnalyzedStatement::Break => format!("{}Break the cycle.", prefix),
        AnalyzedStatement::Continue => format!("{}Continue to the next iteration.", prefix),
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                format!("{}Return with the offering {}.", prefix, tell_expr(v))
            } else {
                format!("{}Return with nothing.", prefix)
            }
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
            let params_str: Vec<String> = params
                .iter()
                .map(|(n, t)| {
                    let type_str = t.as_ref().map(tell_type).unwrap_or("unknown".to_string());
                    format!("`{}` ({})", n, type_str)
                })
                .collect();
            let ret_str = return_type
                .as_ref()
                .map(tell_type)
                .unwrap_or("nothing".to_string());
            let mut s = format!(
                "{}Define a ritual called `{}` expecting [{}] which returns {}:\n",
                prefix,
                name,
                params_str.join(", "),
                ret_str
            );
            for stmt in body {
                s.push_str(&tell_statement(stmt, level + 1));
                s.push('\n');
            }
            s
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let fields_str: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("`{}` as {}", n, tell_type(t)))
                .collect();
            format!(
                "{}Declare a new form `{}` with attributes: {}.",
                prefix,
                name,
                fields_str.join(", ")
            )
        }
        AnalyzedStatement::TraitDefinition { name, methods: _ } => {
            format!("{}Declare a characteristic named `{}`.", prefix, name)
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods: _,
        } => {
            format!(
                "{}Imbue `{}` with the characteristic of `{}`.",
                prefix, type_name, trait_name
            )
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            let mut s = format!("{}Define a trial named `{}`:\n", prefix, name);
            for stmt in body {
                s.push_str(&tell_statement(stmt, level + 1));
                s.push('\n');
            }
            s
        }
    }
}

fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("the text \"{}\"", s),
        AnalyzedExprKind::NumberLiteral(n) => format!("the number {}", n),
        AnalyzedExprKind::BooleanLiteral(b) => format!("the truth {}", b),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("the `{}` of {}", property, tell_expr(owner))
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("{}ing [{}]", verb, args_str.join(", "))
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            format!("({} {:?} {})", tell_expr(left), op, tell_expr(right))
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            format!("({:?} {})", op, tell_expr(operand))
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let range_op = if *inclusive { "through" } else { "up to" };
            format!("from {} {} {}", tell_expr(start), range_op, tell_expr(end))
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            let expr_strs: Vec<String> = exprs.iter().map(tell_expr).collect();
            format!("a list containing [{}]", expr_strs.join(", "))
        }
        AnalyzedExprKind::Some(e) => format!("something ({})", tell_expr(e)),
        AnalyzedExprKind::None => "nothing".to_string(),
        AnalyzedExprKind::Ok(e) => format!("success ({})", tell_expr(e)),
        AnalyzedExprKind::Err(e) => format!("failure ({})", tell_expr(e)),
        AnalyzedExprKind::Unwrap(e) => format!("the essence of {}", tell_expr(e)),
        AnalyzedExprKind::Try(e) => format!("attempting {}", tell_expr(e)),
        AnalyzedExprKind::IndexAccess { array, index } => {
            format!("the item at {} in {}", tell_expr(index), tell_expr(array))
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("calling `{}` with [{}]", func, args_str.join(", "))
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!(
                "invoking `{}` on {} with [{}]",
                method,
                tell_expr(receiver),
                args_str.join(", ")
            )
        }
        AnalyzedExprKind::TraitMethodCall {
            receiver,
            trait_name,
            method_name,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!(
                "invoking `{}` (as `{}`) on {} with [{}]",
                method_name,
                trait_name,
                tell_expr(receiver),
                args_str.join(", ")
            )
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!(
                "a new `{}` with fields [{}] set to [{}]",
                type_name,
                fields.join(", "),
                args_str.join(", ")
            )
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            let mode = match capture_mode {
                CaptureMode::Borrow => "borrowing",
                CaptureMode::Move => "moving",
                CaptureMode::Memoize => "remembering",
            };
            format!(
                "a spirit {} [{}] that produces {}",
                mode,
                params.join(", "),
                tell_expr(body)
            )
        }
        AnalyzedExprKind::CollectionNew { collection_type } => {
            format!("a new empty {}", collection_type)
        }
        AnalyzedExprKind::Assert { condition } => {
            format!("asserting that {} is true", tell_expr(condition))
        }
        AnalyzedExprKind::AssertEq { left, right } => format!(
            "asserting that {} equals {}",
            tell_expr(left),
            tell_expr(right)
        ),
    }
}

fn tell_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "Number".to_string(),
        GlossaType::String => "Text".to_string(),
        GlossaType::Boolean => "Truth".to_string(),
        GlossaType::List(inner) => format!("List of {}", tell_type(inner)),
        GlossaType::Set(inner) => format!("Set of {}", tell_type(inner)),
        GlossaType::Map(k, v) => format!("Map from {} to {}", tell_type(k), tell_type(v)),
        GlossaType::Option(inner) => format!("Maybe {}", tell_type(inner)),
        GlossaType::Result(ok, err) => format!("Result of {} or {}", tell_type(ok), tell_type(err)),
        GlossaType::Struct { name, .. } => format!("Form `{}`", name),
        GlossaType::Function { params, returns } => {
            let params_str: Vec<String> = params.iter().map(tell_type).collect();
            format!(
                "Function({}) -> {}",
                params_str.join(", "),
                tell_type(returns)
            )
        }
        GlossaType::Unit => "Nothing".to_string(),
        GlossaType::Unknown => "Mystery".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_bard_basic() {
        let source = "ξ πέντε ἔστω.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let tale = tell_tale(&analyzed);

        assert!(tale.contains("Let there be a variable named `ξ` with the value the number 5."));
    }

    #[test]
    fn test_bard_print() {
        let source = "«χαῖρε» λέγε.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let tale = tell_tale(&analyzed);

        assert!(tale.contains("Proclaim to the world: the text \"χαῖρε\"."));
    }
}
