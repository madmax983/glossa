//! Semantic validation and limits
//!
//! This module enforces semantic rules and resource limits on the analyzed AST.
//! It ensures that the generated program is safe to compile and execute, preventing
//! DoS attacks like stack overflows from deeply nested expressions.

use super::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::errors::GlossaError;
use crate::limits::MAX_EXPRESSION_DEPTH;

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
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::GlossaType;

    #[test]
    fn test_statement_depth_limit() {
        let stmt = AnalyzedStatement::Break;
        let result = check_statement_depth(&stmt, MAX_EXPRESSION_DEPTH + 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            GlossaError::LimitExceeded { resource, max } => {
                assert_eq!(resource, "statement depth");
                assert_eq!(max, MAX_EXPRESSION_DEPTH);
            }
            _ => panic!("Expected LimitExceeded error"),
        }
    }

    #[test]
    fn test_expr_depth_limit() {
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: GlossaType::Unknown,
        };
        let result = check_expr_depth(&expr, MAX_EXPRESSION_DEPTH + 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            GlossaError::LimitExceeded { resource, max } => {
                assert_eq!(resource, "expression depth");
                assert_eq!(max, MAX_EXPRESSION_DEPTH);
            }
            _ => panic!("Expected LimitExceeded error"),
        }
    }

    #[test]
    fn test_return_statement_depth() {
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: GlossaType::Unknown,
        };
        let stmt = AnalyzedStatement::Return {
            value: Some(Box::new(expr)),
        };
        let result = check_statement_depth(&stmt, 0);
        assert!(result.is_ok());
    }
}
#[cfg(test)]
mod additional_tests {
    use crate::semantic::{
        AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType,
        validation::{check_expr_depth, check_statement_depth},
    };
    use smol_str::SmolStr;

    #[test]
    fn test_expr_depth_coverage() {
        let dummy_expr = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: GlossaType::Unknown,
        });

        // Test Some
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::Some(dummy_expr.clone()),
            glossa_type: GlossaType::Unknown,
        };
        assert!(check_expr_depth(&expr, 0).is_ok());

        // Test UnaryOp
        let expr2 = AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp {
                op: crate::morphology::UnaryOp::Not,
                operand: dummy_expr.clone(),
            },
            glossa_type: GlossaType::Unknown,
        };
        assert!(check_expr_depth(&expr2, 0).is_ok());

        // Test IndexAccess
        let expr3 = AnalyzedExpr {
            expr: AnalyzedExprKind::IndexAccess {
                array: dummy_expr.clone(),
                index: dummy_expr.clone(),
            },
            glossa_type: GlossaType::Unknown,
        };
        assert!(check_expr_depth(&expr3, 0).is_ok());

        // Test ArrayLiteral
        let expr4 = AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(vec![*dummy_expr.clone()]),
            glossa_type: GlossaType::Unknown,
        };
        assert!(check_expr_depth(&expr4, 0).is_ok());

        // Test StructInstantiation
        let expr5 = AnalyzedExpr {
            expr: AnalyzedExprKind::StructInstantiation {
                type_name: SmolStr::new("Test"),
                fields: vec![SmolStr::new("x")],
                args: vec![*dummy_expr.clone()],
            },
            glossa_type: GlossaType::Unknown,
        };
        assert!(check_expr_depth(&expr5, 0).is_ok());
    }

    #[test]
    fn test_statement_depth_coverage() {
        let dummy_expr = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::None,
            glossa_type: GlossaType::Unknown,
        });
        let dummy_stmt = AnalyzedStatement::Break;

        // Test While
        let stmt = AnalyzedStatement::While {
            condition: dummy_expr.clone(),
            body: vec![dummy_stmt.clone()],
        };
        assert!(check_statement_depth(&stmt, 0).is_ok());

        // Test For
        let stmt2 = AnalyzedStatement::For {
            iterator: dummy_expr.clone(),
            variable: SmolStr::new("x"),
            body: vec![dummy_stmt.clone()],
        };
        assert!(check_statement_depth(&stmt2, 0).is_ok());

        // Test Match
        let stmt3 = AnalyzedStatement::Match {
            scrutinee: dummy_expr.clone(),
            arms: vec![(*dummy_expr.clone(), vec![dummy_stmt.clone()])],
        };
        assert!(check_statement_depth(&stmt3, 0).is_ok());
    }
}
