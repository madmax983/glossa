# 16. Extract Semantic Analyzer

Date: 2026-03-29

## Status

Accepted

## Context

The `src/semantic/mod.rs` module was previously acting as the orchestrator for semantic analysis while simultaneously depending on submodules like `control_flow.rs` and `declarations.rs`. These submodules, in turn, depended on the `analyze_statement` function defined in `mod.rs`. This created a circular dependency where the module structure became fragile, tightly coupled, and difficult to maintain or extend.

## Decision

We extracted the core semantic analysis logic from `src/semantic/mod.rs` into a new, dedicated `src/semantic/analyzer.rs` module.

The `src/semantic/mod.rs` file now acts as a clean facade, re-exporting the `analyze_program` API from `analyzer.rs` to maintain backward compatibility, while `analyzer.rs` handles the orchestration logic, directly calling `declarations`, `control_flow`, and other submodules without cyclic dependencies back to `mod.rs`.

## Consequences

*   **Broken Dependency Cycle:** The dependency graph is now a Directed Acyclic Graph (DAG). `mod.rs` simply orchestrates and exports the public API, while `analyzer.rs` interacts with the submodules. Submodules are now leaf-like with respect to the orchestrator.
*   **Improved Cohesion:** The logic for analyzing statements recursively is isolated in `analyzer.rs`, separating it from the high-level orchestration setup in `mod.rs`.
*   **Maintainability:** Changes to the analysis logic no longer risk breaking the module boundaries defined in `mod.rs`.
