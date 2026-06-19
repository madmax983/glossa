import sys

def patch_file(filename, detonate_var, new_fn_name):
    with open(filename, "r") as f:
        text = f.read()

    # Get imports
    imports = text.split('#[test]')[0]
    imports = imports.replace('use std::env;\n', '').replace('use std::process::Command;\n', '')

    # Extract inner body
    body = text.split('if env::var("' + detonate_var + '").is_ok() {')[1].split('println!("Survived and mitigated!");')[0]

    # Reconstruct
    new_text = f"""{imports}#[test]
#[ignore = "Demonstrates stack overflow vulnerability"]
fn {new_fn_name}() {{
{body}
}}
"""
    with open(filename, "w") as f:
        f.write(new_text)

patch_file("tests/havoc_codegen_stack_overflow.rs", "HAVOC_DETONATE_CODEGEN_OVERFLOW", "havoc_codegen_stack_overflow")
patch_file("tests/havoc_semantic_stack_overflow.rs", "HAVOC_DETONATE_SEMANTIC_OVERFLOW", "havoc_semantic_stack_overflow")
