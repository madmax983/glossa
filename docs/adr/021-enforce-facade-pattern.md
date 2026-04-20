# 021. Enforce Facade Pattern in src/lib.rs

Date: 2026-05-18

## Status

Accepted

## Context

The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`). This leaked implementation details and created a sprawling public API, violating the principle of encapsulation and making it difficult for downstream users to know which functions to use.

## Decision

We have refactored `src/lib.rs` to change these internal modules to `pub(crate) mod` (or kept `pub mod` only where explicitly needed by the `glossa` binary or integration tests) and added explicit `pub use` statements for the true public API:
- `ast::Program`
- `codegen::generate_rust`
- `parser::parse`
- `semantic::{AnalyzedProgram, analyze_program}`

## Consequences

- **Encapsulation:** Creates a clean "Facade" that hides messy internal sub-modules.
- **Usability:** Exposes only what the user needs, making the public API more intentional and easier to navigate.
- **Coupling:** Reduces tight coupling between external consumers and the crate's internal module layout.
