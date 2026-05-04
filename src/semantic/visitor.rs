use super::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};

/// A trait for recursively traversing the semantic AST.
/// By default, the methods in this trait will recursively visit all child nodes.
/// Implementors can override specific methods to intercept the traversal at interesting points.
pub trait Visitor {
    fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        walk_statement(self, stmt);
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr) {
        walk_expr(self, expr);
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, stmt: &AnalyzedStatement) {
    match stmt {
        AnalyzedStatement::Binding { value, .. } => {
            visitor.visit_expr(value);
        }
        AnalyzedStatement::Assignment { value, .. } => {
            visitor.visit_expr(value);
        }
        AnalyzedStatement::Print(exprs) => {
            for expr in exprs {
                visitor.visit_expr(expr);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            for expr in exprs {
                visitor.visit_expr(expr);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            for expr in exprs {
                visitor.visit_expr(expr);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            visitor.visit_expr(condition);
            for s in then_body {
                visitor.visit_statement(s);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    visitor.visit_statement(s);
                }
            }
        }
        AnalyzedStatement::While { condition, body } => {
            visitor.visit_expr(condition);
            for s in body {
                visitor.visit_statement(s);
            }
        }
        AnalyzedStatement::For { iterator, body, .. } => {
            visitor.visit_expr(iterator);
            for s in body {
                visitor.visit_statement(s);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            visitor.visit_expr(scrutinee);
            for (expr, stmts) in arms {
                visitor.visit_expr(expr);
                for s in stmts {
                    visitor.visit_statement(s);
                }
            }
        }
        AnalyzedStatement::FunctionDef {
            params: _,
            body,
            return_type: _,
            name: _,
        } => {
            for s in body {
                visitor.visit_statement(s);
            }
        }
        AnalyzedStatement::TestDeclaration { body, name: _ } => {
            for s in body {
                visitor.visit_statement(s);
            }
        }
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                visitor.visit_expr(v);
            }
        }
        AnalyzedStatement::TraitImplementation {
            methods,
            trait_name: _,
            type_name: _,
        } => {
            for method in methods {
                if let Some(body) = &method.body {
                    for s in body {
                        visitor.visit_statement(s);
                    }
                }
            }
        }
        AnalyzedStatement::Break
        | AnalyzedStatement::Continue
        | AnalyzedStatement::TypeDefinition { .. }
        | AnalyzedStatement::TraitDefinition { .. } => {}
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(visitor: &mut V, expr: &AnalyzedExpr) {
    match &expr.expr {
        AnalyzedExprKind::PropertyAccess { owner, .. } => {
            visitor.visit_expr(owner);
        }
        AnalyzedExprKind::VerbCall { args, .. } => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => {
            visitor.visit_expr(operand);
        }
        AnalyzedExprKind::StructInstantiation { args, .. } => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            visitor.visit_expr(receiver);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        AnalyzedExprKind::FunctionCall { args, .. } => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            for expr in exprs {
                visitor.visit_expr(expr);
            }
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            visitor.visit_expr(array);
            visitor.visit_expr(index);
        }
        AnalyzedExprKind::Lambda { body, .. } => {
            visitor.visit_expr(body);
        }
        AnalyzedExprKind::Some(inner) => visitor.visit_expr(inner),
        AnalyzedExprKind::Ok(inner) => visitor.visit_expr(inner),
        AnalyzedExprKind::Err(inner) => visitor.visit_expr(inner),
        AnalyzedExprKind::Unwrap(inner) => visitor.visit_expr(inner),
        AnalyzedExprKind::Try(inner) => visitor.visit_expr(inner),
        AnalyzedExprKind::Assert { condition } => visitor.visit_expr(condition),
        AnalyzedExprKind::AssertEq { left, right } => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        AnalyzedExprKind::Range { start, end, .. } => {
            visitor.visit_expr(start);
            visitor.visit_expr(end);
        }
        AnalyzedExprKind::Variable(_)
        | AnalyzedExprKind::NumberLiteral(_)
        | AnalyzedExprKind::StringLiteral(_)
        | AnalyzedExprKind::BooleanLiteral(_)
        | AnalyzedExprKind::None
        | AnalyzedExprKind::CollectionNew { .. } => {}
    }
}
