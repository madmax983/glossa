import re

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

# I removed mutation_count and mutable_vars from audit_expr, but I left them in the tests!
# We just need to remove them from the test setup entirely if they aren't used.
content = content.replace("        let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();\n        let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();\n", "")

with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
