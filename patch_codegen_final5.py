import re
with open('src/codegen.rs', 'r') as f:
    content = f.read()

new_test = """
    #[test]
    fn test_generate_unreachable_operators_all_unreachable_branches_all_ops() {
        let left = AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral("left".to_string()),
            glossa_type: GlossaType::String,
        };
        let right = AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral("right".to_string()),
            glossa_type: GlossaType::String,
        };

        // If the types are NOT Number, they hit the match op fallthrough.
        let ops = vec![
            (BinaryOp::Add, "+"),
            (BinaryOp::Sub, "-"),
            (BinaryOp::Mul, "*"),
            (BinaryOp::Div, "/"),
            (BinaryOp::Mod, "%"),
            (BinaryOp::Eq, "=="),
            (BinaryOp::Ne, "!="),
        ];

        for (op, expected) in ops {
            let tokens = generate_bin_op(op, &left, &right);
            assert!(tokens.to_string().contains(expected));
        }
    }
"""
content = re.sub(r'#\[test\]\n\s*fn test_generate_unreachable_operators_all_unreachable_branches\(\) \{.*?\}\n\s*\}', new_test, content, flags=re.DOTALL)
with open('src/codegen.rs', 'w') as f:
    f.write(content)
