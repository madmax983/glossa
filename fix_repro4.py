import re

with open("tests/havoc_repro.rs", "r") as f:
    code = f.read()

search_block = """    let res = execute_script_to_string(&source);
    if let Ok(out) = res {
        // Just assert it compiles/runs, or error is fine if it's missing verb
    }"""

replace_block = """    let res = execute_script_to_string(&source);
    // Ignore the result. Havoc's job here is just to cause chaos."""

code = code.replace(search_block, replace_block)

with open("tests/havoc_repro.rs", "w") as f:
    f.write(code)
