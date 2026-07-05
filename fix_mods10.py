import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("tests/havoc_operator_stress.rs", "glossa::semantic::AssemblyError::LimitExceeded", "glossa::semantic::assembly::AssemblyError::LimitExceeded")
