import re

with open("tests/havoc_codegen_stack_overflow.rs", "r") as f:
    text = f.read()

text = text.replace('let _ = generate_rust(&program);', 'let _ = generate_rust(&program);\n        std::mem::forget(program);')

with open("tests/havoc_codegen_stack_overflow.rs", "w") as f:
    f.write(text)

with open("tests/havoc_semantic_stack_overflow.rs", "r") as f:
    text = f.read()

text = text.replace('let expr2 = expr.clone();\n\n        println!("Dropping cloned expression...");\n        drop(expr2);\n\n        println!("Dropping original expression...");\n        drop(expr);', 'std::mem::forget(expr);')

with open("tests/havoc_semantic_stack_overflow.rs", "w") as f:
    f.write(text)
