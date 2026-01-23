//! High-level Intermediate Representation
//!
//! A simplified IR that maps closely to Rust constructs.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement, StatementKind, AnalyzedExpr, AnalyzedExprKind};

/// A HIR program ready for code generation
#[derive(Debug, Clone)]
pub struct HirProgram {
    pub statements: Vec<HirStatement>,
}

/// A HIR statement
#[derive(Debug, Clone)]
pub enum HirStatement {
    /// let name = value;
    Let {
        name: String,
        value: HirExpr,
        mutable: bool,
    },

    /// println!(...);
    Print {
        args: Vec<HirExpr>,
    },

    /// expression;
    Expr(HirExpr),
}

/// A HIR expression
#[derive(Debug, Clone)]
pub enum HirExpr {
    /// String literal
    StringLit(String),

    /// Integer literal
    IntLit(i64),

    /// Boolean literal
    BoolLit(bool),

    /// Variable reference
    Var(String),

    /// Field access: expr.field
    Field {
        object: Box<HirExpr>,
        field: String,
    },

    /// Method call: expr.method(args)
    MethodCall {
        receiver: Box<HirExpr>,
        method: String,
        args: Vec<HirExpr>,
    },

    /// Function call: func(args)
    Call {
        func: String,
        args: Vec<HirExpr>,
    },

    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },

    /// Reference: &expr or &mut expr
    Ref {
        mutable: bool,
        expr: Box<HirExpr>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Lower analyzed program to HIR
pub fn lower_to_hir(analyzed: &AnalyzedProgram) -> HirProgram {
    let mut statements = Vec::new();

    for stmt in &analyzed.statements {
        if let Some(hir_stmt) = lower_statement(stmt) {
            statements.push(hir_stmt);
        }
    }

    HirProgram { statements }
}

fn lower_statement(stmt: &AnalyzedStatement) -> Option<HirStatement> {
    match &stmt.kind {
        StatementKind::Binding { name, .. } => {
            // Get the value expression (second expression in the list)
            let value = if stmt.expressions.len() > 1 {
                lower_expr(&stmt.expressions[1])
            } else {
                HirExpr::IntLit(0) // Default
            };

            Some(HirStatement::Let {
                name: name.clone(),
                value,
                mutable: false,
            })
        }

        StatementKind::Print => {
            let args: Vec<HirExpr> = stmt.expressions.iter()
                .map(|e| lower_expr(e))
                .collect();

            Some(HirStatement::Print { args })
        }

        StatementKind::Expression => {
            if let Some(first) = stmt.expressions.first() {
                Some(HirStatement::Expr(lower_expr(first)))
            } else {
                None
            }
        }

        StatementKind::Query => {
            // For now, queries become print statements
            let args: Vec<HirExpr> = stmt.expressions.iter()
                .map(|e| lower_expr(e))
                .collect();

            Some(HirStatement::Print { args })
        }
    }
}

fn lower_expr(expr: &AnalyzedExpr) -> HirExpr {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => HirExpr::StringLit(s.clone()),
        AnalyzedExprKind::NumberLiteral(n) => HirExpr::IntLit(*n),
        AnalyzedExprKind::BooleanLiteral(b) => HirExpr::BoolLit(*b),
        AnalyzedExprKind::Variable(name) => HirExpr::Var(name.clone()),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            HirExpr::Field {
                object: Box::new(lower_expr(owner)),
                field: property.clone(),
            }
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            HirExpr::Call {
                func: verb.clone(),
                args: args.iter().map(|a| lower_expr(a)).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::build_ast;
    use crate::semantic::analyze_program;

    #[test]
    fn test_lower_hello() {
        let ast = build_ast("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let hir = lower_to_hir(&analyzed);

        assert_eq!(hir.statements.len(), 1);
        assert!(matches!(hir.statements[0], HirStatement::Print { .. }));
    }

    #[test]
    fn test_lower_binding() {
        let ast = build_ast("ξ πέντε ἔστω.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let hir = lower_to_hir(&analyzed);

        assert!(matches!(
            &hir.statements[0],
            HirStatement::Let { name, .. } if name == "ξ"
        ));
    }

    #[test]
    fn test_lower_number_literal() {
        let ast = build_ast("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let hir = lower_to_hir(&analyzed);

        if let HirStatement::Print { args } = &hir.statements[0] {
            assert!(matches!(args[0], HirExpr::IntLit(42)));
        } else {
            panic!("Expected Print statement");
        }
    }
}
