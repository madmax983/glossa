# 12. Consolidate Compiler Tools

Date: 2025-02-12
Status: Accepted

## Context

As the ΓΛΩΣΣΑ compiler grew, the root source directory (`src/`) became cluttered with auxiliary tool modules that were distinct from the core compilation pipeline (Parsing -> Semantic Analysis -> Code Generation). These modules included:
- `highlight` (Syntax highlighting)
- `report` (Compilation reports)
- `repl` (Interactive playground)
- `cli` (Command-line interface)
- `runner` (Compilation pipeline orchestration)
- `cache` (Incremental compilation)

Keeping these modules at the top level made it harder to distinguish between the core compiler logic and the tooling ecosystem built around it. It also obscured the dependency graph, as tools naturally depend on the core compiler components.

## Decision

We have consolidated all auxiliary tool modules into a dedicated `src/tools/` directory.

- `src/tools/mod.rs`: Serves as the new entry point for the toolset.
- `src/highlight.rs` -> `src/tools/highlight.rs`
- `src/report.rs` -> `src/tools/report.rs`
- `src/repl.rs` -> `src/tools/repl.rs`
- `src/cli.rs` -> `src/tools/cli.rs`
- `src/runner.rs` -> `src/tools/runner.rs`
- `src/cache.rs` -> `src/tools/cache.rs`

(Note: `narrator` was previously moved to `src/tools/narrator.rs` in ADR 010, establishing the precedent for this consolidation.)

## Consequences

- **Structural Clarity**: The root `src/` directory now clearly separates core compiler modules (`parser`, `morphology`, `semantic`, `codegen`) from the tooling ecosystem (`tools`).
- **Module Organization**: The `tools` module acts as a namespace for all user-facing utilities.
- **Improved DX**: Developers can easily locate tool-specific logic in one place.
- **Documentation**: Architecture diagrams must be updated to reflect the new paths.
