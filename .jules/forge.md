**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**[Clippy Refactoring: Disambiguation Contexts]
**Learning:** We need to keep our functions small and readable. `analyze_article` in `src/morphology/disambiguation.rs` was a giant `match` block returning `Some(DisambiguationContext { ... })` and triggered `clippy::too_many_lines`.
**Action:** Extract a helper function `ctx(case: Option<Case>, num: Option<Number>, gen: Option<Gender>) -> Option<DisambiguationContext>` to remove the boilerplate from `analyze_article`. (Initially used `impl Into<Option<T>>` but it caused inference errors when passing `None`, so explicit `Option<T>` is safer).
