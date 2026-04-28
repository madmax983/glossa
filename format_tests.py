import re

content = open("src/semantic/control_flow.rs").read()

test_imports_match = re.search(r'(    use super::\*;.*?    use crate::ast::.*?;)', content, re.DOTALL)
if test_imports_match:
    imports = test_imports_match.group(1)

    # We remove it and place it right after `mod tests {`
    content = content.replace(imports, "")
    content = content.replace("mod tests {", "mod tests {\n" + imports + "\n")

    # Write back
    open("src/semantic/control_flow.rs", "w").write(content)
