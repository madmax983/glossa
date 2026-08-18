**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored CLI Command Matcher**
**Learning:** The `main` function was becoming a God Function due to a massive `match` statement handling dozens of CLI commands, exacerbated by duplicated `#[cfg(feature = "nova")]` compiler directives in every experimental command arm.
**Action:** Extracted the routing logic into `execute_command` and `execute_nova_command` helper functions, and grouped feature-gated logic into single conditionally compiled helpers to reduce repetition and maintain high diff coverage.
