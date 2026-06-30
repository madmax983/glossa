with open("tests/razor_coverage.rs", "r") as f:
    content = f.read()

target = '    assert!(format!("{}", err_undefined).contains("Ἄγνωστον ὄνομα"));'
replacement = '    assert!(format!("{}", err_undefined).contains("Οὐκ οἶδα τὸ ὄνομα"));'

if target in content:
    content = content.replace(target, replacement)
    print("Patched razor correctly")
else:
    print("Target not found")

with open("tests/razor_coverage.rs", "w") as f:
    f.write(content)
