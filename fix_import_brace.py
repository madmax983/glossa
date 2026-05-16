import os

files = ['src/semantic/conversion/statements.rs', 'src/semantic/conversion/values.rs', 'src/semantic/conversion/tests.rs']

for filename in files:
    with open(filename, 'r') as f:
        content = f.read()

    # We extracted imports using lines from `mod.rs`.
    # Wait, the `use crate::semantic::expressions::{ ...` block was split across lines!
    # In `src/semantic/conversion.rs`, it was:
    # use super::expressions::{
    #     analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    #     literal_to_analyzed_expr, literal_to_type,
    # };
    # Our script only grabbed `use super::` lines, so it grabbed:
    # use super::expressions::{
    # and completely missed the inner lines and the `};`.

    content = content.replace('use crate::semantic::expressions::{', 'use crate::semantic::expressions::{\n    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,\n    literal_to_analyzed_expr, literal_to_type,\n};')

    with open(filename, 'w') as f:
        f.write(content)
