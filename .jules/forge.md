**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Codegen Program Partitioning**
**Learning:** Found manual iterating logic duplicating memory and pushing to multiple vecs dynamically.
**Action:** Replaced with a struct that initializes `with_capacity` correctly and avoids manual partitioning overhead via an `add_statement` method.
