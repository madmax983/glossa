import re

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

# I removed them from tests but test_auditor_visitor_coverage_statements calls audit_statement which STILL needs them!
# So we must put them back in the tests, but prefix them with underscore OR just not remove them from audit_statement, but they are used in audit_statement so it's fine.

content = content.replace("let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();\n",
                          "let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();\n        let mut mutation_count = FxHashMap::default();\n        let mut mutable_vars = FxHashSet::default();\n")

# test_auditor_visitor_coverage_expressions calls audit_expr which ONLY needs usage_count now.
content = content.replace("audit_expr(&expr, &mut usage_count, &mut mutation_count, &mut mutable_vars);", "audit_expr(&expr, &mut usage_count);")


with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
