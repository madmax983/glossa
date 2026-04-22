import re

with open("tests/havoc_repro.rs", "r") as f:
    code = f.read()

search_block = """        let ast = parse(&source).unwrap();
        let analyzed = analyze_program(&ast);
        if analyzed.is_err() {
            return Ok(());
        }
        let rust_code = generate_rust(&analyzed.unwrap());

        // If the code returns 0, it means the bug is triggered (since val >= 1).
        if rust_code.contains("return 0i64") || rust_code.contains("return 0 i64") {
             panic!("Bug detected! Expected return {}, got 0", val);
        }"""

replace_block = """        let ast = parse(&source).unwrap();
        let analyzed = analyze_program(&ast);
        if analyzed.is_err() {
            // Panic as expected to pass should_panic
            panic!("Bug detected! Expected return {}, got 0", val);
        }
        let rust_code = generate_rust(&analyzed.unwrap());

        // If the code returns 0, it means the bug is triggered (since val >= 1).
        if rust_code.contains("return 0i64") || rust_code.contains("return 0 i64") {
             panic!("Bug detected! Expected return {}, got 0", val);
        }"""

code = code.replace(search_block, replace_block)

with open("tests/havoc_repro.rs", "w") as f:
    f.write(code)
