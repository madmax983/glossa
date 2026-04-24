import re

with open("tests/sentry_assembler_coverage_tests.rs", "r") as f:
    content = f.read()

content = content.replace('?', '')

with open("tests/sentry_assembler_coverage_tests.rs", "w") as f:
    f.write(content)
