import re

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

# I accidentally duplicated variable declarations during the replacement.
content = content.replace(
"""    let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
        let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
        let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();
    let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
    let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();""",
"""    let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
    let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
    let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();"""
)

with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
