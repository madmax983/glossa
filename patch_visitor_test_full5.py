import os
with open('src/semantic/visitor.rs', 'r') as f:
    content = f.read()

# Add a comprehensive test to hit all variants of walk_statement and walk_expr
test_code = """
#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    use crate::semantic::{GlossaType, AnalyzedMethod, CaptureMode};
    use crate::morphology::lexicon::{BinaryOp, UnaryOp};

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
        let mut visitor = DummyVisitor { visited_statements: 0, visited_exprs: 0 };

        let stmts = vec![
            AnalyzedStatement::Binding { name: "x".into(), value: dummy_expr(), mutable: false },
            AnalyzedStatement::Assignment { name: "x".into(), value: dummy_expr() },
            AnalyzedStatement::Print(vec![dummy_expr()]),
            AnalyzedStatement::Expression(vec![dummy_expr()]),
            AnalyzedStatement::Query(vec![dummy_expr()]),
            AnalyzedStatement::If {
                condition: Box::new(dummy_expr()),
                then_body: vec![],
                else_body: Some(vec![]),
            },
            AnalyzedStatement::While { condition: Box::new(dummy_expr()), body: vec![] },
            AnalyzedStatement::For { variable: "x".into(), iterator: Box::new(dummy_expr()), body: vec![] },
            AnalyzedStatement::Match { scrutinee: Box::new(dummy_expr()), arms: vec![(dummy_expr(), vec![])] },
            AnalyzedStatement::FunctionDef { name: "f".into(), params: vec![], body: vec![], return_type: None },
            AnalyzedStatement::TestDeclaration { name: "t".into(), body: vec![] },
            AnalyzedStatement::Return { value: Some(Box::new(dummy_expr())) },
            AnalyzedStatement::Return { value: None },
            AnalyzedStatement::Break,
            AnalyzedStatement::Continue,
            AnalyzedStatement::TypeDefinition { name: "T".into(), fields: vec![] },
            AnalyzedStatement::TraitDefinition { name: "Tr".into(), methods: vec![] },
            AnalyzedStatement::TraitImplementation {
                trait_name: "Tr".into(),
                type_name: "T".into(),
                methods: vec![AnalyzedMethod { name: "m".into(), params: vec![], body: Some(vec![]), return_type: None }]
            },
        ];

        for stmt in stmts {
            visitor.visit_statement(&stmt);
        }

        let exprs = vec![
            AnalyzedExprKind::Variable("x".into()),
            AnalyzedExprKind::PropertyAccess { owner: Box::new(dummy_expr()), property: "p".into() },
            AnalyzedExprKind::VerbCall { verb: "v".into(), args: vec![dummy_expr()] },
            AnalyzedExprKind::BinOp { left: Box::new(dummy_expr()), op: BinaryOp::Add, right: Box::new(dummy_expr()) },
            AnalyzedExprKind::UnaryOp { op: UnaryOp::Not, operand: Box::new(dummy_expr()) },
            AnalyzedExprKind::StructInstantiation { type_name: "T".into(), fields: vec![], args: vec![dummy_expr()] },
            AnalyzedExprKind::MethodCall { receiver: Box::new(dummy_expr()), method: "m".into(), args: vec![dummy_expr()] },
            AnalyzedExprKind::FunctionCall { func: "f".into(), args: vec![dummy_expr()] },
            AnalyzedExprKind::ArrayLiteral(vec![dummy_expr()]),
            AnalyzedExprKind::IndexAccess { array: Box::new(dummy_expr()), index: Box::new(dummy_expr()) },
            AnalyzedExprKind::Lambda { params: vec![], body: Box::new(dummy_expr()), capture_mode: CaptureMode::Borrow },
            AnalyzedExprKind::Some(Box::new(dummy_expr())),
            AnalyzedExprKind::Ok(Box::new(dummy_expr())),
            AnalyzedExprKind::Err(Box::new(dummy_expr())),
            AnalyzedExprKind::Unwrap(Box::new(dummy_expr())),
            AnalyzedExprKind::Try(Box::new(dummy_expr())),
            AnalyzedExprKind::Assert { condition: Box::new(dummy_expr()) },
            AnalyzedExprKind::AssertEq { left: Box::new(dummy_expr()), right: Box::new(dummy_expr()) },
            AnalyzedExprKind::Range { start: Box::new(dummy_expr()), end: Box::new(dummy_expr()), inclusive: false },
            AnalyzedExprKind::CollectionNew { collection_type: "HashSet".into() },
            AnalyzedExprKind::NumberLiteral(1),
            AnalyzedExprKind::StringLiteral("hello".to_string()),
            AnalyzedExprKind::BooleanLiteral(true),
            AnalyzedExprKind::None,
        ];

        for kind in exprs {
            visitor.visit_expr(&AnalyzedExpr { expr: kind, glossa_type: GlossaType::Unknown });
        }
    }
}
"""

with open('src/semantic/visitor.rs', 'w') as f:
    f.write(content + "\n" + test_code)
