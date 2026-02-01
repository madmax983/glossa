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
