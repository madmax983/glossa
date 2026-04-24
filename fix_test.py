import re

with open("tests/word_order_tests.rs", "r") as f:
    content = f.read()

content = content.replace('    let _analyzed = analyze_program(&ast).expect("Should analyze");', '    let res = analyze_program(&ast);\n    assert!(res.is_err());')

with open("tests/word_order_tests.rs", "w") as f:
    f.write(content)
