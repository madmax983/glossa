import re
import os

with open("src/semantic/conversion.rs", "r") as f:
    lines = f.readlines()

def get_block(lines, start_idx):
    start_of_doc = start_idx
    for i in range(start_idx - 1, -1, -1):
        if lines[i].strip().startswith("///") or lines[i].strip().startswith("#[allow("):
            start_of_doc = i
        else:
            break

    open_braces = 0
    in_fn = False
    for j in range(start_idx, len(lines)):
        l = lines[j]
        if "{" in l:
            in_fn = True
            open_braces += l.count("{")
        if "}" in l:
            open_braces -= l.count("}")
        if in_fn and open_braces == 0:
            return start_of_doc, j
    return start_idx, -1

# Functions to extract to extraction.rs
extract_fns = [
    "fn extract_unwrap(",
    "fn extract_enum_from_subject(",
    "fn extract_genitive_method(",
    "fn extract_enum_from_nominatives(",
    "fn extract_property_access(",
    "fn extract_index_access(",
    "fn extract_array(",
    "fn extract_binary_op(",
    "fn extract_literal(",
    "fn extract_enum_from_object(",
    "fn extract_object_fallback(",
    "pub fn extract_value(",
    "fn resolve_binding_target(",
]

# Functions to extract to classification.rs
classify_fns = [
    "pub fn classify_assembled_statement(",
    "fn classify_iterator_pattern(",
    "fn classify_property_access_print(",
    "fn classify_function_call(",
    "fn resolve_function_name(",
    "fn classify_subjunctive_comparison(",
    "fn classify_variable_binding(",
    "fn classify_assignment(",
    "fn classify_collection_mutation(",
    "fn classify_pop(",
    "fn classify_push(",
    "fn classify_insert(",
    "fn classify_assertion(",
    "fn classify_equality_assertion(",
    "fn try_print_binary_op(",
    "fn try_print_property_access(",
    "fn try_print_index_access(",
    "fn try_print_unwrap(",
    "fn try_print_default(",
    "fn classify_print(",
    "fn classify_query(",
    "fn classify_containment_query(",
    "fn classify_expression(",
    "fn try_parse_genitive_method_call(",
    "fn classify_genitive_method_call(",
    "fn detect_enum_variant(",
]

def find_line(lines, prefix):
    for i, line in enumerate(lines):
        if line.startswith(prefix):
            return i
    return -1

def create_module(lines, fns_list, name):
    extracted_indices = set()
    code = ""
    for prefix in fns_list:
        idx = find_line(lines, prefix)
        if idx != -1:
            start_idx, end_idx = get_block(lines, idx)
            for i in range(start_idx, end_idx + 1):
                # Make functions pub(crate) if they are just fn
                line = lines[i]
                if i == idx and line.startswith("fn "):
                    line = "pub(crate) " + line
                elif i == idx and line.startswith("pub fn "):
                    line = "pub " + line[7:]
                code += line
                extracted_indices.add(i)
    return code, extracted_indices

extraction_code, extraction_indices = create_module(lines, extract_fns, "extraction")
classification_code, classification_indices = create_module(lines, classify_fns, "classification")

# Create directories and files
os.makedirs("src/semantic/conversion", exist_ok=True)

# Common imports for the new files
common_imports = """use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::morphology::{self};
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;
use crate::semantic::{Constituent, Literal};
use crate::semantic::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr, literal_to_type,
};
use crate::semantic::patterns::detect_iterator_pattern;
use super::classification::*;
use super::extraction::*;

"""

with open("src/semantic/conversion/extraction.rs", "w") as f:
    f.write(common_imports)
    f.write(extraction_code)

with open("src/semantic/conversion/classification.rs", "w") as f:
    f.write(common_imports)
    f.write(classification_code)

# Now, generate mod.rs
mod_lines = []
for i, line in enumerate(lines):
    if i not in extraction_indices and i not in classification_indices:
        mod_lines.append(line)

mod_content = "".join(mod_lines)

# Fix up the mod.rs
# Add the module declarations at the top, after the doc comments
mod_res = []
in_doc = True
for line in mod_lines:
    if in_doc and not line.startswith("//!") and line.strip() != "":
        in_doc = False
        mod_res.append("pub(crate) mod classification;\n")
        mod_res.append("pub mod extraction;\n\n")
        mod_res.append("pub use classification::*;\n")
        mod_res.append("pub use extraction::*;\n\n")
    mod_res.append(line)

with open("src/semantic/conversion/mod.rs", "w") as f:
    f.writelines(mod_res)
