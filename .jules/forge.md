**[Redundant Formatting Borrow]
**Learning:** `clippy::useless_borrows_in_formatting` is triggered when passing a reference to a variable directly inside `format!` arguments, e.g., `format!("{}", &var)`.
**Action:** Always use the variable directly, `format!("{}", var)`, unless passing the reference is required for a specific trait implementation not implemented on the base type.
