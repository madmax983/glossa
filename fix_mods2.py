import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("src/semantic/mod.rs", "pub use assembly::{\n    AssembledStatement, AssemblyError, Constituent, Literal, ParticipleConstituent, VerbConstituent,\n};", "pub(crate) use assembly::{\n    AssembledStatement, AssemblyError, Constituent, Literal, ParticipleConstituent, VerbConstituent,\n};")
replace_in_file("src/semantic/mod.rs", "pub use assembly::Assembler;", "pub(crate) use assembly::Assembler;")
replace_in_file("tests/razor_coverage.rs", "use glossa::semantic::{", "use glossa::semantic::assembly::{model::{")
replace_in_file("tests/razor_coverage.rs", "AssembledStatement, Constituent, Literal, ParticipleConstituent, VerbConstituent,", "AssembledStatement, Constituent, Literal, ParticipleConstituent, VerbConstituent}}; use glossa::semantic::{")
