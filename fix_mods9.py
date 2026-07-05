import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("tests/semantic_model_coverage_tests.rs", "TraitDef, TraitImpl}; use glossa::semantic::assembly::model::{AssembledStatement, Constituent, Literal, ParticipleConstituent, VerbConstituent};\n};", "TraitDef, TraitImpl}; use glossa::semantic::assembly::model::{AssembledStatement, Constituent, Literal, ParticipleConstituent, VerbConstituent};")
