import re

with open('tests/warden_coverage.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'fn compile(source: &str) {',
    'fn compile(source: &str) {\n    println!("COMPILING: {}", source);'
)

with open('tests/warden_coverage.rs', 'w') as f:
    f.write(content)
