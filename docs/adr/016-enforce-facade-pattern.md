# 16. Enforce Facade Pattern in Public API

Date: 2026-04-05
Status: Accepted

## Context

The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`). This leaked implementation details and created a sprawling public API, violating the principle of encapsulation and making it difficult for downstream users to know which functions to use.

## Decision

We have refactored `src/lib.rs` to change these internal modules to `pub(crate) mod` (keeping `pub mod` only where explicitly needed by the `glossa` binary or integration tests). We then added explicit `pub use` statements for the true public API: `ast::Program`, `codegen::generate_rust`, `parser::parse`, and `semantic::{AnalyzedProgram, analyze_program}`.

This establishes a clean "Facade" pattern that hides messy internal sub-modules while exposing only the components the user actually needs.

## Consequences

*   **Encapsulation:** Internal implementation details of parsing, semantics, and morphology are safely hidden from consumers.
*   **API Clarity:** The compiler's public interface is concise and discoverable, pointing clearly to the main parsing, analysis, and codegen pipeline functions.
*   **Maintenance:** Internal structural changes are much less likely to break downstream code or integration tests.
