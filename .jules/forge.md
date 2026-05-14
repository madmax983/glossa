**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Flattened conditional CLI match arms**
**Learning:** Conditional feature flags inside match arms lead to ugly fallback code (like `let _ = var`) to suppress unused variable warnings.
**Action:** Apply `#[cfg]` attributes to the entire match arm and use struct wildcards (`{ .. }`) in the negative feature fallback to cleanly discard unused variables without boilerplate.
