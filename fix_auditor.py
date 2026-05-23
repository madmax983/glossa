with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

# To fix this `clippy::only_used_in_recursion` warning, we can just allow it on `audit_expr` since this is part of a mutually recursive visiting pattern and it's cleaner to pass all state along even if one function only uses it to pass it back to another, or in this case, to itself.
# Oh actually, `audit_expr` never calls `audit_statement`! It only calls `audit_expr`.
# And since it only calls `audit_expr`, and `audit_expr` never uses `mutation_count` or `mutable_vars` directly, they are truly "only used in recursion". We can just drop them from `audit_expr`!
# Let's see: `audit_expr` takes: expr, usage_count, mutation_count, mutable_vars.
# Let's remove `mutation_count` and `mutable_vars` from `audit_expr` completely.

content = content.replace("""pub fn audit_expr(
    expr: &AnalyzedExpr,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
)""", """pub fn audit_expr(
    expr: &AnalyzedExpr,
    usage_count: &mut FxHashMap<SmolStr, usize>,
)""")

import re
content = re.sub(
    r'audit_expr\(([^,]+),\s*usage_count,\s*mutation_count,\s*mutable_vars\)',
    r'audit_expr(\1, usage_count)',
    content
)

# Also need to update the signature in `audit_statement` calls
content = re.sub(
    r'audit_expr\(([^,]+),\s*&mut\s*usage_count,\s*&mut\s*mutation_count,\s*&mut\s*mutable_vars\)',
    r'audit_expr(\1, &mut usage_count)',
    content
)

with open('src/tools/auditor.rs', 'w') as f:
    f.write(content)
