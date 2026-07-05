import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("src/semantic/assembly/mod.rs", "use glossa::semantic::assembly::Assembler;", "use glossa::semantic::Assembler;")
replace_in_file("src/semantic/conversion.rs", "use glossa::semantic::assembly::AssembledStatement;", "use glossa::semantic::AssembledStatement;")
