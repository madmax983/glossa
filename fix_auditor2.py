import re

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

# Fix test type inference
content = content.replace(
    "let mut mutation_count = FxHashMap::default();",
    "let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();"
)
content = content.replace(
    "let mut mutable_vars = FxHashSet::default();",
    "let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();"
)
content = content.replace(
    "let mut usage_count = FxHashMap::default();",
    "let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();"
)

with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
