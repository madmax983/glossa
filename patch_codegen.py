import re

with open("src/codegen.rs", "r") as f:
    text = f.read()

new_expr = r"""fn generate_expr(expr: &AnalyzedExpr) -> TokenStream {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || match &expr.expr {
"""

text = re.sub(r'fn generate_expr\(expr: &AnalyzedExpr\) -> TokenStream \{\n    match &expr.expr \{', new_expr, text)

text = re.sub(
    r'        AnalyzedExprKind::AssertEq \{ left, right \} => generate_control_assert_eq\(left, right\),\n    \}',
    '        AnalyzedExprKind::AssertEq { left, right } => generate_control_assert_eq(left, right),\n    })',
    text
)

new_stmt = r"""fn generate_statement(stmt: &AnalyzedStatement) -> TokenStream {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || match stmt {
"""

text = re.sub(r'fn generate_statement\(stmt: &AnalyzedStatement\) -> TokenStream \{\n    match stmt \{', new_stmt, text)

text = re.sub(
    r'        AnalyzedStatement::TestDeclaration \{ name, body \} => generate_test\(name, body\),\n    \}',
    '        AnalyzedStatement::TestDeclaration { name, body } => generate_test(name, body),\n    })',
    text
)

with open("src/codegen.rs", "w") as f:
    f.write(text)
