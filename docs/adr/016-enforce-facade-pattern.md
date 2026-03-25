# 16. Enforce Facade Pattern in src/lib.rs

Date: 2025-03-25
Status: Accepted

## Context

The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`). This leaked implementation details and created a sprawling public API, violating the principle of encapsulation and making it difficult for downstream users to know which functions and structures to use. A clean module boundary was lacking, forcing users to navigate the internal complexity of the compiler pipeline.

## Decision

We have enforced the Facade Pattern in `src/lib.rs`.

Internal modules have been changed to `pub(crate) mod` (or kept `pub mod` only where explicitly needed by the `glossa` binary or integration tests). We added explicit `pub use` statements to form the true public API:
* `ast::Program`
* `codegen::generate_rust`
* `parser::parse`
* `semantic::{AnalyzedProgram, analyze_program}`
* `tools::highlight`

## Consequences

### Positive
- **Encapsulation**: Hides messy internal sub-modules from the public interface.
- **Usability**: Creates a clean, discoverable "Facade" exposing only what the downstream user needs to compile and analyze code.
- **Stability**: Internal structural changes (like renaming or splitting submodules) are less likely to break the public API, as long as the facade exports remain stable.

### Negative
- **Integration Test Verbosity**: Some deeply nested integration tests or external diagnostic tools may require explicit `pub mod` exemptions or restructuring to access internal details if they bypass the facade.
