import os

with open('src/semantic/conversion.rs', 'r') as f:
    pass # Wait, we don't have it anymore!

# Fortunately, we can just use `cargo check --tests` to find exactly what's wrong with `tests.rs`.
# In Rust, a test file `tests.rs` is included using `mod tests;`.
# The functions we are trying to import are `pub(crate)` in `statements` and `values` modules.
# Since `tests` is a module inside `conversion` (sibling to `statements` and `values`),
# `use crate::semantic::conversion::statements::*` SHOULD WORK.
# Let's verify why it doesn't work.
