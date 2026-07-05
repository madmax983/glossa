import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("tests/semantic_model_coverage_tests.rs", "AssembledStatement,", "")
replace_in_file("tests/semantic_model_coverage_tests.rs", "Constituent, Literal, ParticipleConstituent, TraitDef, TraitImpl, VerbConstituent,", "TraitDef, TraitImpl}; use glossa::semantic::assembly::model::{AssembledStatement, Constituent, Literal, ParticipleConstituent, VerbConstituent};")
