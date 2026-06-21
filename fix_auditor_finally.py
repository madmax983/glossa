import re

with open("src/tools/auditor.rs", "r") as f:
    content = f.read()

content = content.replace(
"""        let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
        let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();""",
"""        let mut mutation_count = FxHashMap::default();
        let mut mutable_vars = FxHashSet::default();"""
)


content = content.replace(
"""    let mut usage_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
    let mut mutation_count: FxHashMap<SmolStr, usize> = FxHashMap::default();
    let mut mutable_vars: FxHashSet<SmolStr> = FxHashSet::default();""",
"""    let mut usage_count = FxHashMap::default();
    let mut mutation_count = FxHashMap::default();
    let mut mutable_vars = FxHashSet::default();"""
)

with open("src/tools/auditor.rs", "w") as f:
    f.write(content)
