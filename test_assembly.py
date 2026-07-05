with open("src/semantic/mod.rs", "r") as f:
    content = f.read()
content = content.replace("pub mod assembly;", "pub(crate) mod assembly;")
with open("src/semantic/mod.rs", "w") as f:
    f.write(content)
