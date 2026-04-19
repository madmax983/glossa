with open("src/tools/alchemist.rs", "r") as f:
    content = f.read()

# I removed the `test_transpile_unimplemented_expr_fallback` earlier, maybe it was incomplete?
# I'll add a fully working dummy test to just cover transpile_expr fallback

test_code = """
    #[test]
    fn test_transpile_unimplemented_expr_fallback() {
        let expr = AnalyzedExpr {
            expr: AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: crate::semantic::GlossaType::Number,
            })),
            glossa_type: crate::semantic::GlossaType::Unknown,
        };
        let py = transpile_expr(&expr);
        assert!(py.contains("/* Unimplemented expr: "));
    }
"""

if "test_transpile_unimplemented_expr_fallback" not in content:
    content = content.replace("mod tests {\n    use super::*;", "mod tests {\n    use super::*;\n" + test_code)

with open("src/tools/alchemist.rs", "w") as f:
    f.write(content)
