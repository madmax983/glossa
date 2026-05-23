import sys

with open("src/tools/scribe.rs", "r") as f:
    content = f.read()

# I will append more tests to the `tests` module in `scribe.rs`
new_tests = """
    #[test]
    fn test_scribe_coverage() {
        use crate::semantic::{AnalyzedExprKind, AnalyzedStatement, AnalyzedExpr, GlossaType};
        use smol_str::SmolStr;

        let mut program = crate::semantic::AnalyzedProgram::new();

        // Construct expressions
        let num_expr = AnalyzedExpr { expr: AnalyzedExprKind::NumberLiteral(42), glossa_type: GlossaType::Number };
        let str_expr = AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("test".to_string()), glossa_type: GlossaType::String };
        let bool_expr = AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(true), glossa_type: GlossaType::Boolean };
        let var_expr = AnalyzedExpr { expr: AnalyzedExprKind::Variable(SmolStr::new("x")), glossa_type: GlossaType::Number };

        let arr_expr = AnalyzedExpr { expr: AnalyzedExprKind::ArrayLiteral(vec![num_expr.clone()]), glossa_type: GlossaType::List(Box::new(GlossaType::Number)) };

        let bin_op_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::BinOp { left: Box::new(num_expr.clone()), op: crate::morphology::lexicon::BinaryOp::Add, right: Box::new(num_expr.clone()) },
            glossa_type: GlossaType::Number
        };

        let un_op_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp { op: crate::morphology::lexicon::UnaryOp::Not, operand: Box::new(bool_expr.clone()) },
            glossa_type: GlossaType::Boolean
        };

        let prop_acc_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess { owner: Box::new(var_expr.clone()), property: SmolStr::new("prop") },
            glossa_type: GlossaType::Number
        };

        let meth_call_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::MethodCall { receiver: Box::new(var_expr.clone()), method: SmolStr::new("meth"), args: vec![] },
            glossa_type: GlossaType::Number
        };

        let struct_inst = AnalyzedExpr {
            expr: AnalyzedExprKind::StructInstantiation { type_name: SmolStr::new("Struct"), args: vec![], fields: vec![] },
            glossa_type: GlossaType::Number
        };

        let verb_call = AnalyzedExpr {
            expr: AnalyzedExprKind::VerbCall { verb: SmolStr::new("verb"), args: vec![] },
            glossa_type: GlossaType::Number
        };

        // Add statements
        program.statements.push(AnalyzedStatement::Binding { name: SmolStr::new("a"), value: num_expr.clone(), mutable: true });
        program.statements.push(AnalyzedStatement::Assignment { name: SmolStr::new("a"), value: str_expr.clone() });
        program.statements.push(AnalyzedStatement::Print(vec![str_expr.clone()]));
        program.statements.push(AnalyzedStatement::Return { value: Some(num_expr.clone()) });
        program.statements.push(AnalyzedStatement::Return { value: None });

        program.statements.push(AnalyzedStatement::If {
            condition: bool_expr.clone(),
            then_body: vec![AnalyzedStatement::Break],
            else_body: Some(vec![AnalyzedStatement::Continue]),
            is_expression: false,
        });

        program.statements.push(AnalyzedStatement::While { condition: bool_expr.clone(), body: vec![AnalyzedStatement::Break] });
        program.statements.push(AnalyzedStatement::For { variable: SmolStr::new("i"), iterator: arr_expr.clone(), body: vec![AnalyzedStatement::Continue] });

        program.statements.push(AnalyzedStatement::TypeDefinition { name: SmolStr::new("MyType"), fields: vec![(SmolStr::new("f"), GlossaType::Number)] });
        program.statements.push(AnalyzedStatement::TraitDefinition { name: SmolStr::new("MyTrait"), methods: vec![] });
        program.statements.push(AnalyzedStatement::TraitImplementation { trait_name: SmolStr::new("MyTrait"), type_name: SmolStr::new("MyType"), methods: vec![] });

        program.statements.push(AnalyzedStatement::FunctionDef { name: SmolStr::new("f"), params: vec![], body: vec![AnalyzedStatement::Break], return_type: Some(GlossaType::Number), pure: false });
        program.statements.push(AnalyzedStatement::TestDeclaration { name: SmolStr::new("test"), body: vec![] });
        program.statements.push(AnalyzedStatement::Expression(vec![bin_op_expr.clone(), un_op_expr.clone(), prop_acc_expr.clone(), meth_call_expr.clone(), struct_inst.clone(), verb_call.clone()]));

        let json = super::program_to_json(&program);
        assert!(json.contains("Binding"));
        assert!(json.contains("Assignment"));
        assert!(json.contains("Print"));
        assert!(json.contains("Return"));
        assert!(json.contains("If"));
        assert!(json.contains("While"));
        assert!(json.contains("For"));
        assert!(json.contains("TypeDefinition"));
        assert!(json.contains("TraitDefinition"));
        assert!(json.contains("TraitImplementation"));
        assert!(json.contains("FunctionDef"));
        assert!(json.contains("TestDeclaration"));
        assert!(json.contains("Expression"));

        assert!(json.contains("ArrayLiteral"));
        assert!(json.contains("BinOp"));
        assert!(json.contains("UnaryOp"));
        assert!(json.contains("PropertyAccess"));
        assert!(json.contains("MethodCall"));
        assert!(json.contains("StructInstantiation"));
        assert!(json.contains("VerbCall"));
    }
"""

content = content.replace("    fn test_scribe_basic() {", new_tests + "\n    #[test]\n    fn test_scribe_basic() {")

with open("src/tools/scribe.rs", "w") as f:
    f.write(content)
