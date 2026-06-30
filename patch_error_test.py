with open("src/errors/mod.rs", "r") as f:
    content = f.read()

target = '        assert!(err.to_string().contains("Ἄγνωστον ὄνομα"));'
replacement = '        assert!(err.to_string().contains("Οὐκ οἶδα τὸ ὄνομα"));'

if target in content:
    content = content.replace(target, replacement)
    print("Patched target")
else:
    print("Not found")

with open("src/errors/mod.rs", "w") as f:
    f.write(content)
