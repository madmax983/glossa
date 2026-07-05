import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("src/semantic/assembly/mod.rs", "pub use model::*;", "pub(crate) use model::*;")
replace_in_file("src/semantic/mod.rs", "pub mod assembly;", "pub(crate) mod assembly;")
