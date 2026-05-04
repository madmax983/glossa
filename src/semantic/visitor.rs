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

#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    use crate::morphology::lexicon::{BinaryOp, UnaryOp};
    use crate::semantic::{AnalyzedMethod, CaptureMode, GlossaType};

    struct DummyVisitor {
        visited_statements: usize,
        visited_exprs: usize,
    }

    impl Visitor for DummyVisitor {
        fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
            self.visited_statements += 1;
            walk_statement(self, stmt);
        }

        fn visit_expr(&mut self, expr: &AnalyzedExpr) {
            self.visited_exprs += 1;
            walk_expr(self, expr);
        }
    }

    fn dummy_expr() -> AnalyzedExpr {
        AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        }
    }

    #[test]
    fn test_visitor_comprehensive() {
        let mut visitor = DummyVisitor {
            visited_statements: 0,
            visited_exprs: 0,
        };

        let stmts = vec![
            AnalyzedStatement::Binding {
                name: "x".into(),
                value: dummy_expr(),
                mutable: false,
            },
            AnalyzedStatement::Assignment {
                name: "x".into(),
                value: dummy_expr(),
            },
            AnalyzedStatement::Print(vec![dummy_expr()]),
            AnalyzedStatement::Expression(vec![dummy_expr()]),
            AnalyzedStatement::Query(vec![dummy_expr()]),
            AnalyzedStatement::If {
                condition: Box::new(dummy_expr()),
                then_body: vec![],
                else_body: Some(vec![]),
            },
            AnalyzedStatement::While {
                condition: Box::new(dummy_expr()),
                body: vec![],
            },
            AnalyzedStatement::For {
                variable: "x".into(),
                iterator: Box::new(dummy_expr()),
                body: vec![],
            },
            AnalyzedStatement::Match {
                scrutinee: Box::new(dummy_expr()),
                arms: vec![(dummy_expr(), vec![])],
            },
            AnalyzedStatement::FunctionDef {
                name: "f".into(),
                params: vec![],
                body: vec![],
                return_type: None,
            },
            AnalyzedStatement::TestDeclaration {
                name: "t".into(),
                body: vec![],
            },
            AnalyzedStatement::Return {
                value: Some(Box::new(dummy_expr())),
            },
            AnalyzedStatement::Return { value: None },
            AnalyzedStatement::Break,
            AnalyzedStatement::Continue,
            AnalyzedStatement::TypeDefinition {
                name: "T".into(),
                fields: vec![],
            },
            AnalyzedStatement::TraitDefinition {
                name: "Tr".into(),
                methods: vec![],
            },
            AnalyzedStatement::TraitImplementation {
                trait_name: "Tr".into(),
                type_name: "T".into(),
                methods: vec![AnalyzedMethod {
                    name: "m".into(),
                    params: vec![],
                    body: Some(vec![]),
                    return_type: None,
                }],
            },
        ];

        for stmt in stmts {
            visitor.visit_statement(&stmt);
        }

        let exprs = vec![
            AnalyzedExprKind::Variable("x".into()),
            AnalyzedExprKind::PropertyAccess {
                owner: Box::new(dummy_expr()),
                property: "p".into(),
            },
            AnalyzedExprKind::VerbCall {
                verb: "v".into(),
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::BinOp {
                left: Box::new(dummy_expr()),
                op: BinaryOp::Add,
                right: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::StructInstantiation {
                type_name: "T".into(),
                fields: vec![],
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::MethodCall {
                receiver: Box::new(dummy_expr()),
                method: "m".into(),
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::FunctionCall {
                func: "f".into(),
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::ArrayLiteral(vec![dummy_expr()]),
            AnalyzedExprKind::IndexAccess {
                array: Box::new(dummy_expr()),
                index: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::Lambda {
                params: vec![],
                body: Box::new(dummy_expr()),
                capture_mode: CaptureMode::Borrow,
            },
            AnalyzedExprKind::Some(Box::new(dummy_expr())),
            AnalyzedExprKind::Ok(Box::new(dummy_expr())),
            AnalyzedExprKind::Err(Box::new(dummy_expr())),
            AnalyzedExprKind::Unwrap(Box::new(dummy_expr())),
            AnalyzedExprKind::Try(Box::new(dummy_expr())),
            AnalyzedExprKind::Assert {
                condition: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::AssertEq {
                left: Box::new(dummy_expr()),
                right: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::Range {
                start: Box::new(dummy_expr()),
                end: Box::new(dummy_expr()),
                inclusive: false,
            },
            AnalyzedExprKind::CollectionNew {
                collection_type: "HashSet".into(),
            },
            AnalyzedExprKind::NumberLiteral(1),
            AnalyzedExprKind::StringLiteral("hello".to_string()),
            AnalyzedExprKind::BooleanLiteral(true),
            AnalyzedExprKind::None,
        ];

        for kind in exprs {
            visitor.visit_expr(&AnalyzedExpr {
                expr: kind,
                glossa_type: GlossaType::Unknown,
            });
        }
    }
}
