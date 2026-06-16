import re

with open('src/tools/narrator.rs', 'r') as f:
    content = f.read()

# Add doc comment explaining optimization to tell_expr
tell_expr_pattern = r"pub\(crate\) fn tell_expr\(expr: &AnalyzedExpr\) -> String \{"
new_tell_expr_doc = """/// Translates a semantic expression into a readable English string.
///
/// This exists to flatten recursive expression trees (like `AnalyzedExprKind::BinOp`)
/// into linear, human-readable strings. Unlike a standard `Debug` representation
/// which outputs nested structs, this formats operations in a pseudo-code style
/// that is immediately recognizable to developers.
///
/// ⚡ Performance Optimization:
/// To prevent recursive string allocations on deep AST nodes, this wrapper creates
/// a single `String` buffer and delegates to `write_expr`, eliminating intermediate heap allocations.
pub(crate) fn tell_expr(expr: &AnalyzedExpr) -> String {"""
content = re.sub(tell_expr_pattern, new_tell_expr_doc, content)

# Add back doc comment to tell_type and explain optimization
tell_type_pattern = r"fn tell_type\(ty: &GlossaType\) -> String \{"
new_tell_type_doc = """/// Converts a semantic type into a familiar Rust-like type signature string.
///
/// While ΓΛΩΣΣΑ uses Greek terminology internally (e.g., `ἀριθμός`, `λίστη`),
/// the Scroll of Logic translates these into conventional programming type names
/// (e.g., `Number`, `[Type]`) to help developers map the Greek concepts to
/// concepts they already understand.
///
/// ⚡ Performance Optimization:
/// Similar to `tell_expr`, this uses `write_type` with a single mutable buffer
/// to avoid O(N) string allocations during deep recursive type formatting.
fn tell_type(ty: &GlossaType) -> String {"""
content = re.sub(tell_type_pattern, new_tell_type_doc, content)

# Clean up local imports
content = content.replace("use std::fmt::Write;", "")

# Add file level import if not present
if "use std::fmt::Write;" not in content:
    content = "use std::fmt::Write;\n" + content

with open('src/tools/narrator.rs', 'w') as f:
    f.write(content)
