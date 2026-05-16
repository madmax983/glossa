import os
import subprocess

with open('src/semantic/conversion/statements.rs', 'r') as f:
    statements = f.read()

# Make sure tests in tests.rs actually get the functions. The functions are `pub(crate)` in statements.rs.
# wait, statements.rs exports them?
# `pub(crate) fn try_print_binary_op` etc.
# Why isn't `tests.rs` finding them?
# Because `tests.rs` is a sibling module! `crate::semantic::conversion::statements::*` SHOULD find them.
# Let's check `tests.rs` imports block.
