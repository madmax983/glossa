**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored Resolver Iterators & Article Analysis**
**Learning:** `src/semantic/resolver.rs` relied on manual `for level in self.levels.iter().rev() { if let Some(...) = ... return Some(...); }` loops, which are anti-patterns in Rust when simple `.find_map()` exists. Also `src/morphology/disambiguation.rs` heavily duplicated `DisambiguationContext { expected_case: ..., expected_number: ... }` struct literals instead of just using `DisambiguationContext::new()`.
**Action:** Replaced iterative loops with `self.levels.iter().rev().find_map(|level| level.functions.get(name))` and flattened struct literals into a single-line form.
