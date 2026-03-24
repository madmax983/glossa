# 16. Enforce Facade Pattern in glossa crate

Date: 2026-03-24
Status: Accepted

## Context

The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`). This structure leaked internal implementation details to downstream users, creating a sprawling and confusing public API. It violated the principle of encapsulation, making it difficult for users to determine which functions and structures were intended for public consumption and which were strictly internal to the compiler's pipeline.

## Decision

We have enforced the Facade pattern in `src/lib.rs`.

* Internal modules have been downgraded to `pub(crate) mod` (or kept `pub mod` only where explicitly required by the `glossa` binary or integration tests).
* We have added explicit `pub use` statements in `src/lib.rs` to clearly define and export the true public API.

The explicitly exported public API now consists of:
* `tools::highlight`
* `ast::Program`
* `codegen::generate_rust`
* `parser::parse`
* `semantic::{AnalyzedProgram, analyze_program}`

## Consequences

* **Encapsulation**: Internal compiler logic and intermediate structures are hidden from the crate's external interface.
* **Clarity**: Downstream users now have a clean, focused API that exposes only the necessary types and functions for parsing, analyzing, and generating code.
* **Maintainability**: Changing internal module structures is less likely to break the public API, provided the explicit exports in `src/lib.rs` are maintained.
