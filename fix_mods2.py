import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("src/semantic/mod.rs", "pub(crate) use assembly::{\n    AssembledStatement, AssemblyError, Constituent, Literal, ParticipleConstituent, VerbConstituent,\n};", "pub use assembly::{\n    AssembledStatement, AssemblyError, Constituent, Literal, ParticipleConstituent, VerbConstituent,\n};")
replace_in_file("src/semantic/mod.rs", "pub(crate) use assembly::Assembler;", "pub use assembly::Assembler;")
