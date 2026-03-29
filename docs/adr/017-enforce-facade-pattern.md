# 17. Enforce Facade Pattern in src/lib.rs

Date: 2026-03-29

## Status

Accepted

## Context

The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`) via `src/lib.rs`. This leaked compiler implementation details and created a sprawling public API, violating the principles of encapsulation and making it difficult for downstream users to understand which components were meant for public consumption versus internal operation.

## Decision

We enforced the Facade Pattern in `src/lib.rs` by changing internal modules to `pub(crate) mod` (or retaining `pub mod` only where explicitly required by the `glossa` binary or integration tests). We then added explicit `pub use` statements to expose only the true public API of the compiler:
*   `ast::Program`
*   `codegen::generate_rust`
*   `parser::parse`
*   `semantic::{AnalyzedProgram, analyze_program}`
*   `tools::highlight`

## Consequences

*   **Clean Public Interface:** The `glossa` crate now provides a clean, minimalistic facade that hides messy internal submodules while exposing only the necessary API functions to downstream users.
*   **Encapsulation:** Implementation details of internal modules are strictly restricted to the crate, reducing the risk of accidental usage or dependency on unstable internal structures.
*   **API Clarity:** It is now clear which functions and structures represent the supported entry points for parsing, analyzing, and generating Rust code from Glossa sources.
