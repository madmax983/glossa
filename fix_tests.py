import re

with open('src/semantic/conversion/tests.rs', 'r') as f:
    content = f.read()

# Fix the duplicate imports and weird mod structure
content = re.sub(
    r'mod tests \{\n.*?use crate::semantic::conversion::values::\*;\n',
    'mod tests {\n    use super::*;\n    use crate::semantic::conversion::statements::*;\n    use crate::semantic::conversion::values::*;\n    use crate::semantic::{Constituent, Literal, model::AnalyzedExprKind, assembly::AssembledStatement, resolver::Scope, AnalyzedStatement, types::GlossaType};\n',
    content,
    flags=re.DOTALL
)

with open('src/semantic/conversion/tests.rs', 'w') as f:
    f.write(content)
