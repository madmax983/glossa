import re

with open("src/codegen.rs", "r") as f:
    content = f.read()

# Replace the Vec::new() lines in generate_rust
pattern = r"""    let mut trait_defs = Vec::new\(\);
    let mut struct_defs = Vec::new\(\);
    let mut trait_impls = Vec::new\(\);
    let mut fn_defs = Vec::new\(\);
    let mut test_defs = Vec::new\(\);
    let mut main_stmts = Vec::new\(\);"""

replacement = """    // ⚡ Bolt Optimization: Pre-allocate statement vectors to prevent resizing
    // while partitioning the AST nodes during code generation.
    let stmt_len = program.statements.len();
    let mut trait_defs = Vec::with_capacity(stmt_len / 4);
    let mut struct_defs = Vec::with_capacity(stmt_len / 4);
    let mut trait_impls = Vec::with_capacity(stmt_len / 4);
    let mut fn_defs = Vec::with_capacity(stmt_len / 4);
    let mut test_defs = Vec::with_capacity(stmt_len / 4);
    let mut main_stmts = Vec::with_capacity(stmt_len);"""

content = content.replace(pattern, replacement)

with open("src/codegen.rs", "w") as f:
    f.write(content)
