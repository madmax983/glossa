//! Semantic validation and limits
//!
//! This module enforces semantic rules and resource limits on the analyzed AST.
//! It ensures that the generated program is safe to compile and execute, preventing
//! DoS attacks like stack overflows from deeply nested expressions.

use super::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crate::ast::{Clause, Expr, Program, Statement};
use crate::errors::GlossaError;
use crate::limits::{MAX_AST_DEPTH, MAX_EXPRESSION_DEPTH};

/// Validates the raw AST program depth before semantic analysis begins.
pub(crate) fn check_program_depth(program: &Program) -> Result<(), GlossaError> {
    for stmt in &program.statements {
        check_ast_statement_depth(stmt, 0)?;
    }
    Ok(())
}

fn check_ast_statement_depth(stmt: &Statement, depth: usize) -> Result<(), GlossaError> {
    if depth > MAX_AST_DEPTH {
        return Err(GlossaError::semantic(
            "Recursion limit exceeded in statement analysis",
        ));
    }

    match stmt {
        Statement::Regular { clauses, .. } => {
            for clause in clauses {
                check_ast_clause_depth(clause, depth + 1)?;
            }
        }
        Statement::TypeDefinition(_) => {}
        Statement::TraitDefinition(trait_def) => {
            for method in &trait_def.methods {
                if let Some(body) = &method.body {
                    for s in body {
                        check_ast_statement_depth(s, depth + 1)?;
                    }
                }
            }
        }
        Statement::TraitImpl(trait_impl) => {
            for method in &trait_impl.methods {
                for s in &method.body {
                    check_ast_statement_depth(s, depth + 1)?;
                }
            }
        }
        Statement::TestDeclaration(test_decl) => {
            for s in &test_decl.body {
                check_ast_statement_depth(s, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn check_ast_clause_depth(clause: &Clause, depth: usize) -> Result<(), GlossaError> {
    if depth > MAX_AST_DEPTH {
        return Err(GlossaError::semantic(
            "Recursion limit exceeded in clause analysis",
        ));
    }

    for expr in &clause.expressions {
        check_ast_expr_depth(expr, depth + 1)?;
    }
    Ok(())
}

fn check_ast_expr_depth(expr: &Expr, depth: usize) -> Result<(), GlossaError> {
    if depth > MAX_AST_DEPTH {
        return Err(GlossaError::semantic(
            "Recursion limit exceeded in expression analysis",
        ));
    }

    match expr {
        Expr::Phrase(terms) | Expr::ArrayLiteral(terms) => {
            for term in terms {
                check_ast_expr_depth(term, depth + 1)?;
            }
        }
        Expr::PropertyAccess { owner, property } => {
            check_ast_expr_depth(owner, depth + 1)?;
            check_ast_expr_depth(property, depth + 1)?;
        }
        Expr::IndexAccess { array, index } => {
            check_ast_expr_depth(array, depth + 1)?;
            check_ast_expr_depth(index, depth + 1)?;
        }
        Expr::Call { arguments, .. } => {
            for arg in arguments {
                check_ast_expr_depth(arg, depth + 1)?;
            }
        }
        Expr::Binding { value, .. } => check_ast_expr_depth(value, depth + 1)?,
        Expr::BinOp { left, right, .. } => {
            check_ast_expr_depth(left, depth + 1)?;
            check_ast_expr_depth(right, depth + 1)?;
        }
        Expr::UnaryOp { operand, .. } => check_ast_expr_depth(operand, depth + 1)?,
        Expr::Block(statements) => {
            for stmt in statements {
                check_ast_statement_depth(stmt, depth + 1)?;
            }
        }
        Expr::StringLiteral(_)
        | Expr::NumberLiteral(_)
        | Expr::BooleanLiteral(_)
        | Expr::Word(_) => {}
    }
    Ok(())
}

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
    use crate::ast::{
        Clause, ImplMethodDef, TestDecl, TraitDef, TraitImplDef, TraitMethodDecl, Word,
    };
    use crate::semantic::GlossaType;

    fn create_deep_stmt(depth: usize) -> Statement {
        let mut stmt = Statement::Regular {
            clauses: vec![Clause {
                expressions: vec![Expr::Word(Word::new("x"))],
            }],
            is_query: false,
            is_propagate: false,
        };

        for _ in 0..depth {
            stmt = Statement::Regular {
                clauses: vec![Clause {
                    expressions: vec![Expr::Block(vec![stmt])],
                }],
                is_query: false,
                is_propagate: false,
            };
        }
        stmt
    }

    #[test]
    fn test_trait_impl_depth_limit() {
        let deep_stmt = create_deep_stmt(MAX_AST_DEPTH + 1);
        let trait_impl = Statement::TraitImpl(TraitImplDef {
            trait_name: Word::new("Trait"),
            type_name: Word::new("Type"),
            methods: vec![ImplMethodDef {
                name: Word::new("method"),
                params: vec![],
                body: vec![deep_stmt],
            }],
        });

        let result = check_program_depth(&Program {
            statements: vec![trait_impl],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_test_decl_depth_limit() {
        let deep_stmt = create_deep_stmt(MAX_AST_DEPTH + 1);
        let test_decl = Statement::TestDeclaration(TestDecl {
            name: "test".to_string(),
            body: vec![deep_stmt],
        });

        let result = check_program_depth(&Program {
            statements: vec![test_decl],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_trait_def_depth_limit() {
        let deep_stmt = create_deep_stmt(MAX_AST_DEPTH + 1);
        let trait_def = Statement::TraitDefinition(TraitDef {
            name: Word::new("Trait"),
            methods: vec![TraitMethodDecl {
                name: Word::new("method"),
                params: vec![],
                is_default: true,
                body: Some(vec![deep_stmt]),
            }],
        });

        let result = check_program_depth(&Program {
            statements: vec![trait_def],
        });
        assert!(result.is_err());
    }

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
