**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored `try_parse_struct_instantiation`**
**Learning:** Found a god function parsing struct instantiations with multiple nested responsibilities (> 120 lines).
**Action:** Extracted struct pattern verification and collection handling into `verify_struct_instantiation_pattern` and `handle_collection_instantiation` helpers to improve readability.

**Refactored `fmt` in `report.rs`**
**Learning:** Found a large `fmt` function handling multiple distinct layout blocks. Formatting functions table was done inside `fmt` building a complex layout inline.
**Action:** Extracted `format_functions_table` helper, taking the `Formatter` and `AnalyzedProgram` reference. Cleaned up nesting.
