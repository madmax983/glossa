# Razor's Journal 🪒

"Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away." - Antoine de Saint-Exupéry

## [Reduction]
**Bloat:** `src/semantic/agreement.rs` was a standalone module that duplicated agreement logic (which is largely handled by the assembler's `finalize` method) and defined unused error types. It was "Zombie Code" — present but disconnected from the compiler pipeline.
**Cut:** Deleted the entire file and removed its module declaration from `src/semantic/mod.rs`.
**Saved:** ~180 lines of code.

## [Reduction]
**Bloat:** Doc tests in `src/semantic/assembler.rs` were using `String` where `SmolStr` was expected, causing compilation errors during testing. This was technical debt.
**Cut:** Simplified the test setup by using `.into()` to convert strings, making the tests pass and aligning with the actual type usage.
**Saved:** Reduced friction for future developers running tests.

## [Reduction]
**Bloat:** `src/semantic/disambiguation.rs` was a standalone module wrapping simple morphological logic and "fuzzy scoring" that was only used in `expressions.rs`.
**Cut:** Flattened the module. Moved `DisambiguationContext` and `resolve_best` to `src/morphology/mod.rs` (where they logically belong alongside `MorphAnalysis`) and `analyze_article` to `src/morphology/lexicon.rs`.
**Saved:** 1 source file, ~200 lines of module boilerplate and indirection.

## [Complexity]
**Bloat:** `src/semantic/resolver.rs` implemented `Scope` using a recursive `Box<Scope>` structure with a `child()` method that performed a **deep clone** of the entire scope chain. This was O(N^2) copying for nested scopes and allocated excessively on the heap.
**Cut:** Refactored `Scope` to use a flat `Vec<ScopeLevel>` stack. Replaced `child()` with `enter()` and `exit()` methods that modify the stack in-place.
**Saved:** Massive performance win (O(1) scope creation), reduced memory pressure, and simplified the mental model (stack vs recursive tree).
