import sys

with open("src/codegen.rs", "r") as f:
    lines = f.readlines()

def patch(fn_name, match_start, match_end):
    start_idx = -1
    for i, line in enumerate(lines):
        if line.startswith(fn_name):
            start_idx = i
            break

    end_idx = -1
    for i in range(start_idx, len(lines)):
        if match_end in lines[i]:
            end_idx = i + 1
            break

    if start_idx != -1 and end_idx != -1:
        lines[start_idx+1] = f"    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {match_start}\n"
        lines[end_idx] = lines[end_idx].replace("}", "})")

patch("fn generate_expr(", "match &expr.expr {", "AnalyzedExprKind::AssertEq { left, right } => generate_control_assert_eq(left, right),")
patch("fn generate_statement(", "match stmt {", "AnalyzedStatement::TestDeclaration { name, body } => generate_test(name, body),")
patch("fn generate_rust(", "{\n", "format!")

with open("src/codegen.rs", "w") as f:
    f.writelines(lines)
