//! Semantic validation and limits
//!
//! This module enforces semantic rules and resource limits on the analyzed AST.
//! It ensures that the generated program is safe to compile and execute, preventing
//! DoS attacks like stack overflows from deeply nested expressions.

use super::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::errors::GlossaError;

const MAX_EXPRESSION_DEPTH: usize = 200;

/// Validates the analyzed program to ensure it meets all semantic rules and limits.
pub(crate) fn validate_program(program: &AnalyzedProgram) -> Result<(), GlossaError> {
    for stmt in &program.statements {
        check_statement_depth(stmt, 0)?;
    }
    Ok(())
}

fn check_statement_depth(stmt: &AnalyzedStatement, depth: usize) -> Result<(), GlossaError> {
    if depth > MAX_EXPRESSION_DEPTH {
        return Err(GlossaError::LimitExceeded {
            resource: "statement depth".into(),
            max: MAX_EXPRESSION_DEPTH,
        });
    }

    match stmt {
        AnalyzedStatement::Binding { value, .. } | AnalyzedStatement::Assignment { value, .. } => {
            check_expr_depth(value, depth + 1)?;
        }
        AnalyzedStatement::Print(exprs)
        | AnalyzedStatement::Expression(exprs)
        | AnalyzedStatement::Query(exprs) => {
            for expr in exprs {
                check_expr_depth(expr, depth + 1)?;
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            check_expr_depth(condition, depth + 1)?;
            for s in then_body {
                check_statement_depth(s, depth + 1)?;
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    check_statement_depth(s, depth + 1)?;
                }
            }
        }
        AnalyzedStatement::While { condition, body } => {
            check_expr_depth(condition, depth + 1)?;
            for s in body {
                check_statement_depth(s, depth + 1)?;
            }
        }
        AnalyzedStatement::For { iterator, body, .. } => {
            check_expr_depth(iterator, depth + 1)?;
            for s in body {
                check_statement_depth(s, depth + 1)?;
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            check_expr_depth(scrutinee, depth + 1)?;
            for (pat, body) in arms {
                check_expr_depth(pat, depth + 1)?;
                for s in body {
                    check_statement_depth(s, depth + 1)?;
                }
            }
        }
        AnalyzedStatement::FunctionDef { body, .. }
        | AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                check_statement_depth(s, depth + 1)?;
            }
        }
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                check_expr_depth(v, depth + 1)?;
            }
        }
        AnalyzedStatement::Break
        | AnalyzedStatement::Continue
        | AnalyzedStatement::TypeDefinition { .. }
        | AnalyzedStatement::TraitDefinition { .. }
        | AnalyzedStatement::TraitImplementation { .. } => {}
    }
    Ok(())
}

fn check_expr_depth(expr: &AnalyzedExpr, depth: usize) -> Result<(), GlossaError> {
    if depth > MAX_EXPRESSION_DEPTH {
        return Err(GlossaError::LimitExceeded {
            resource: "expression depth".into(),
            max: MAX_EXPRESSION_DEPTH,
        });
    }

    match &expr.expr {
        AnalyzedExprKind::PropertyAccess { owner, .. } => check_expr_depth(owner, depth + 1)?,
        AnalyzedExprKind::VerbCall { args, .. } | AnalyzedExprKind::FunctionCall { args, .. } => {
            for arg in args {
                check_expr_depth(arg, depth + 1)?;
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            check_expr_depth(left, depth + 1)?;
            check_expr_depth(right, depth + 1)?;
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => check_expr_depth(operand, depth + 1)?,
        AnalyzedExprKind::Range { start, end, .. } => {
            check_expr_depth(start, depth + 1)?;
            check_expr_depth(end, depth + 1)?;
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            for e in exprs {
                check_expr_depth(e, depth + 1)?;
            }
        }
        AnalyzedExprKind::Some(e)
        | AnalyzedExprKind::Ok(e)
        | AnalyzedExprKind::Err(e)
        | AnalyzedExprKind::Unwrap(e)
        | AnalyzedExprKind::Try(e) => check_expr_depth(e, depth + 1)?,
        AnalyzedExprKind::IndexAccess { array, index } => {
            check_expr_depth(array, depth + 1)?;
            check_expr_depth(index, depth + 1)?;
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. }
        | AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
            check_expr_depth(receiver, depth + 1)?;
            for arg in args {
                check_expr_depth(arg, depth + 1)?;
            }
        }
        AnalyzedExprKind::StructInstantiation { args, .. } => {
            for arg in args {
                check_expr_depth(arg, depth + 1)?;
            }
        }
        AnalyzedExprKind::Lambda { body, .. } => check_expr_depth(body, depth + 1)?,
        AnalyzedExprKind::Assert { condition } => check_expr_depth(condition, depth + 1)?,
        AnalyzedExprKind::AssertEq { left, right } => {
            check_expr_depth(left, depth + 1)?;
            check_expr_depth(right, depth + 1)?;
        }
        AnalyzedExprKind::StringLiteral(_)
        | AnalyzedExprKind::NumberLiteral(_)
        | AnalyzedExprKind::BooleanLiteral(_)
        | AnalyzedExprKind::Variable(_)
        | AnalyzedExprKind::None
        | AnalyzedExprKind::CollectionNew { .. } => {}
    }
    Ok(())
}
