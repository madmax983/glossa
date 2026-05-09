**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored `to_rust_type`**\n**Learning:** Found a performance smell where `to_rust_type` recursively allocated `String`s using `format!`.\n**Action:** Extracted the logic into a recursive helper `write_rust_type` that writes directly into a mutable `String` buffer using `std::fmt::Write`.
