# 16. Implement Root API Facade

Date: 2026-03-20
Status: Accepted

## Context
The codebase lacked clear module boundaries at the root level, indiscriminately exposing internal implementation details via `pub mod` to downstream consumers. This violated encapsulation and increased the public surface area unnecessarily.

## Decision
Applied the Facade pattern by restricting the visibility of internal modules (`pub(crate) mod` where possible, or keeping them `pub` where tests required) while explicitly lifting the primary public APIs to the top level via targeted `pub use` statements.

## Consequences
### Positive
- Reduced public surface area, ensuring external users only depend on intentionally exported structures like `Program`, `generate_rust`, `parse`, `AnalyzedProgram`, and `analyze_program`.
- Improved encapsulation and clearer boundaries between internal modules and the public API.

### Negative
- Requires maintaining explicit `pub use` statements in `src/lib.rs` for any new public APIs.
