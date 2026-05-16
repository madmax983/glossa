import os
import re

with open('src/semantic/conversion.rs', 'r') as f:
    lines = f.readlines()

os.makedirs('src/semantic/conversion', exist_ok=True)

# Find where extract_value starts
extract_idx = -1
for i, line in enumerate(lines):
    if line.startswith('pub fn extract_value('):
        extract_idx = i
        # move up to grab doc comments
        while extract_idx > 0 and lines[extract_idx - 1].strip().startswith('///'):
            extract_idx -= 1
        break

# Find where tests start
test_idx = -1
for i, line in enumerate(lines):
    if line.startswith('#[cfg(test)]'):
        test_idx = i
        break

# Find where imports end
imports_end = 0
for i, line in enumerate(lines):
    if line.startswith('pub fn convert_assembled_to_analyzed'):
        imports_end = i
        while imports_end > 0 and lines[imports_end - 1].strip().startswith('///'):
            imports_end -= 1
        break

# 1. mod.rs
mod_lines = lines[:imports_end]
mod_content = "".join(mod_lines) + """
pub(crate) mod statements;
pub(crate) mod values;

#[cfg(test)]
mod tests;

pub use statements::convert_assembled_to_analyzed;
pub(crate) use statements::classify_assembled_statement;
pub(crate) use values::extract_value;
"""
with open('src/semantic/conversion/mod.rs', 'w') as f:
    f.write(mod_content)

# 2. Imports block
import_lines = []
for line in lines[mod_lines.count('\n'):imports_end]:
    if line.startswith('use super::'):
        import_lines.append(line.replace('use super::', 'use crate::semantic::'))
    elif line.startswith('use crate::'):
        import_lines.append(line)
import_str = "".join(import_lines)


def make_pub_crate(code):
    # Make all `fn name(` into `pub(crate) fn name(`
    # But only at the start of line (to avoid matching inside closures)
    code = re.sub(r'^fn ([a-zA-Z0-9_]+)', r'pub(crate) fn \1', code, flags=re.MULTILINE)
    return code


# 3. statements.rs
statements_code = "".join(lines[imports_end:extract_idx])
statements_code = make_pub_crate(statements_code)

with open('src/semantic/conversion/statements.rs', 'w') as f:
    f.write(import_str + "\n")
    f.write("pub(crate) use crate::semantic::conversion::values::*;\n\n")
    f.write(statements_code)

# 4. values.rs
values_code = "".join(lines[extract_idx:test_idx])
values_code = make_pub_crate(values_code)

with open('src/semantic/conversion/values.rs', 'w') as f:
    f.write(import_str + "\n")
    f.write("pub(crate) use crate::semantic::conversion::statements::*;\n\n")
    f.write(values_code)

# 5. tests.rs
tests_code = "".join(lines[test_idx:])
tests_code = tests_code.replace('use super::*;', 'use crate::semantic::conversion::statements::*;\n    use crate::semantic::conversion::values::*;')
# tests require extra imports since we split them out
tests_code = tests_code.replace(
    'use crate::semantic::{Constituent, Literal};',
    'use crate::semantic::{Constituent, Literal, model::AnalyzedExprKind, assembly::AssembledStatement, resolver::Scope, AnalyzedStatement, types::GlossaType};\n' +
    'use crate::semantic::expressions::{build_binary_expr, build_expressions_from_literals_and_ops, literal_to_analyzed_expr, literal_to_type, analyze_argument_expr};'
)

with open('src/semantic/conversion/tests.rs', 'w') as f:
    f.write(import_str + "\n")
    f.write(tests_code)
