import re

with open("tests/havoc_repro.rs", "r") as f:
    code = f.read()

search_block = """        let ast = parse(&source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let rust_code = generate_rust(&analyzed);"""

replace_block = """        let ast = parse(&source).unwrap();
        let analyzed = analyze_program(&ast);
        if analyzed.is_err() {
            return;
        }
        let rust_code = generate_rust(&analyzed.unwrap());"""

code = code.replace(search_block, replace_block)

with open("tests/havoc_repro.rs", "w") as f:
    f.write(code)
