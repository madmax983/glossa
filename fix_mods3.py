import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("tests/sentry_assembler_coverage_tests.rs", "use glossa::semantic::Assembler;", "use glossa::semantic::assembly::Assembler;")
replace_in_file("tests/sentry_assembler_coverage_tests.rs", "use glossa::semantic::Literal;", "use glossa::semantic::assembly::model::Literal;")
