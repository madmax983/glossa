import re

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

content = content.replace("let mut mutation_count = FxHashMap::default();", "let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();")
content = content.replace("let mut mutable_vars = FxHashSet::default();", "let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();")

# Now handle the test `test_auditor_visitor_coverage_expressions` where these variables are unused.
# They are declared but never used in `test_auditor_visitor_coverage_expressions` because we removed them from `audit_expr` call.
# Let's just remove them from `test_auditor_visitor_coverage_expressions` specifically.
# It's at the start of that function:
str_to_replace = """    fn test_auditor_visitor_coverage_expressions() {
        let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
        let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
        let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();"""

new_str = """    fn test_auditor_visitor_coverage_expressions() {
        let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();"""

content = content.replace(str_to_replace, new_str)


with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
