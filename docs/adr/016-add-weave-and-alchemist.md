# 016. Add Weave and Alchemist Developer Tools

Date: 2026-03-31
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler ecosystem is expanding to include new tools that enhance the developer experience ("Nova"). We need to provide alternative export and compilation targets to prove the independence of the semantic phase from the Rust code generation phase, and to make it easier to see how Greek syntax maps to semantic meaning and compiled code.

## Decision

We introduced `Weave` (`src/tools/weave.rs`) as a Markdown exporter that generates a 'Rosetta Stone' document combining Glossa source code, semantic assembly logic, and generated Rust code. We also introduced `Alchemist` (ὁ Χημικός) (`src/tools/alchemist.rs`) as an experimental Python transpiler. Both tools are added to the `src/tools/` module and are gated behind the `#[cfg(feature = "nova")]` feature flag in `src/tools/mod.rs` to signify their experimental nature.

## Consequences

### Positive
*   **Improved Developer Experience**: `Weave` provides an excellent educational and documentation tool by showing the mapping between Glossa source, semantics, and Rust code.
*   **Validation of Architecture**: `Alchemist` proves that the semantic analysis phase produces an AST that is independent of the Rust codegen backend, allowing for alternative compilation targets.
*   **Experimentation**: Gating these features behind `nova` allows for safe experimentation without affecting the core compiler stability.

### Negative
*   **Maintenance Overhead**: The addition of these tools increases the maintenance burden. `Alchemist` must be updated whenever the semantic model (`AnalyzedProgram`) changes, and `Weave` depends on both the semantic and codegen phases.
