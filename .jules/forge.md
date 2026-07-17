**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Haruspex visit_expr**
**Learning:** Found a god function visit_expr > 160 lines, handling too many match arms directly inline.
**Action:** Extracted the match arms for Some, None, Ok, Err, Unwrap, and Try into small, named helper functions to flatten the structure and improve readability.
**Fixed Useless Borrow**
**Learning:** Clippy reported useless borrow in format macro.
**Action:** Removed redundant ampersand from var_name in src/semantic/conversion.rs.
