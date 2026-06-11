**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored Report tool visit_statement logic**
**Learning:** Found a Pyramid of Doom with deep nesting inside `TraitImplementation` match arm.
**Action:** Extracted the logic into a separate `visit_methods_body` helper function to flatten the structure using an early return.
