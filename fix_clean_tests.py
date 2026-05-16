import os

with open('src/semantic/conversion/tests.rs', 'r') as f:
    tests_content = f.read()

# tests_content is missing imports
imports = """
use crate::semantic::conversion::statements::*;
use crate::semantic::conversion::values::*;
"""
# insert inside the `mod tests {` block
tests_content = tests_content.replace('mod tests {', f'mod tests {{\n{imports}')

with open('src/semantic/conversion/tests.rs', 'w') as f:
    f.write(tests_content)
