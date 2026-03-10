# 016. Refactor Semantic Analyzer into Orchestrator Structure

Date: 2026-03-10
Status: Accepted

## Context

The `src/semantic/mod.rs` module was orchestrating analysis but also depending on submodules like `control_flow.rs` and `declarations.rs`. These submodules in turn depended on `analyze_statement` from `mod.rs`, creating a circular dependency that made the module structure fragile and tightly coupled.

## Decision

We have decided to break the dependency cycle by extracting the core analysis logic into a new `src/semantic/analyzer.rs` module.

Specifically:
1.  Extracted the analysis logic into a new `src/semantic/analyzer.rs` module with a `SemanticAnalyzer` struct.
2.  Defined a `StatementAnalyzer` trait in `src/semantic/traits.rs` to abstract the recursion.
3.  Updated `control_flow.rs` and `declarations.rs` to accept `&mut impl StatementAnalyzer` instead of calling a concrete function.
4.  Re-exported the public API from `mod.rs` to maintain backward compatibility.

## Consequences

*   **Acyclic Graph:** The circular dependency is broken. The dependency graph is now a DAG: `mod` -> `analyzer` -> `control_flow` -> `traits`. Submodules are now leaf-like with respect to the analyzer.
*   **Clearer Boundaries:** By abstracting the recursive analysis step into a trait (`StatementAnalyzer`), the logic is strictly separated between orchestrating the parsing and analyzing specific patterns.
*   **Backwards Compatibility:** No external API surface was changed since `mod.rs` continues to re-export the required functionality for external consumers.
