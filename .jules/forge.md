**[Extracted CLI command router]
**Learning:** `src/main.rs` had a God Function (`main`) of ~200 lines matching on every CLI command with feature gates.
**Action:** Extracted it into `execute_command` and `execute_nova_command` to flatten the structure and keep main tiny.

**[Extracted CLI command router]**
**Learning:** `src/main.rs` had a God Function (`main`) of ~200 lines matching on every CLI command with feature gates.
**Action:** Extracted it into `execute_command` and `execute_nova_command` to flatten the structure and keep main tiny.

**[Removed redundant borrow in format!]**
**Learning:** Passing a reference `&var_name` inside `format!()` when `var_name` is already a string slice/owned string triggers the `clippy::useless_borrows_in_formatting` lint.
**Action:** Removed the redundant `&` to satisfy Clippy's strict warnings (`-D warnings`).
