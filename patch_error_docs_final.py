with open("src/errors/mod.rs", "r") as f:
    content = f.read()

target = """    #[error("Ἄγνωστον ὄνομα: {name}")]"""
replacement = """    #[error("Οὐκ οἶδα τὸ ὄνομα: {name}")]"""

if target in content:
    content = content.replace(target, replacement)
    print("Patched target")

with open("src/errors/mod.rs", "w") as f:
    f.write(content)
