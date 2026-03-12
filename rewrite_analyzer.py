import re

with open('src/semantic/analyzer.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'use super::\{AnalyzedStatement, GlossaType, Scope, StatementAnalyzer, assemble_statement\};',
    r'use super::{AnalyzedStatement, GlossaType, Scope, assemble_statement};',
    content
)

content = re.sub(r'/// The Semantic Analyzer orchestrates the semantic analysis process\.\npub struct SemanticAnalyzer;\n\nimpl SemanticAnalyzer \{\n    pub fn new\(\) -> Self \{\n        Self\n    \}\n\}\n\nimpl Default for SemanticAnalyzer \{\n    fn default\(\) -> Self \{\n        Self::new\(\)\n    \}\n\}\n\nimpl StatementAnalyzer for SemanticAnalyzer \{', r'', content, flags=re.DOTALL)

with open('src/semantic/analyzer.rs', 'w') as f:
    f.write(content)
