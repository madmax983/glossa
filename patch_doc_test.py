import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

target = """/// use glossa::semantic::assembly::Assembler;
"""
replacement = """/// use glossa::semantic::Assembler;
"""

if target in content:
    content = content.replace(target, replacement)
    with open("src/semantic/assembly/mod.rs", "w") as f:
        f.write(content)
    print("Replaced Assembler successfully")
else:
    print("Target not found")
