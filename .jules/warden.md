**2026-07-26 - Useless Borrow in Formatting macro**
**Threat:** A redundant reference in the `format!` argument was triggering `clippy::useless_borrows_in_formatting`.
**Defense:** Replaced the redundant `&var_name` with `var_name` inside `format!`.
