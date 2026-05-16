import os

with open('src/semantic/conversion/tests.rs', 'r') as f:
    content = f.read()

imports = """
use crate::semantic::conversion::statements::*;
use crate::semantic::conversion::values::*;
use crate::semantic::{Constituent, Literal, model::AnalyzedExprKind, assembly::AssembledStatement, resolver::Scope, AnalyzedStatement, types::GlossaType};
"""
# insert right after mod tests {
content = content.replace('mod tests {', 'mod tests {\n' + imports)

with open('src/semantic/conversion/tests.rs', 'w') as f:
    f.write(content)
