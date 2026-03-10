# 16. Break Semantic Module Dependency Cycle

Date: 2026-03-07
Status: Proposed

## Context

The `semantic` module contained a circular dependency between the main orchestrator (`analyzer.rs`) and specialized submodules (`control_flow.rs`, `declarations.rs`). These submodules directly depended on the concrete `Analyzer` implementation in order to recursively parse statements within loops, conditionals, and functions. This tight coupling complicated testing and violated dependency graph rules (creating an acyclic graph violation).

## Decision

We introduced a `StatementAnalyzer` trait in a new `traits.rs` module to abstract the statement analysis recursion logic. The semantic logic was extracted into a `SemanticAnalyzer` struct that implements this trait. The specialized submodules (`control_flow.rs` and `declarations.rs`) now rely on the `StatementAnalyzer` trait instead of the concrete orchestrator implementation.

## Consequences

### Positive
- **Acyclic Dependency Graph**: Breaking the circular dependency enforces a clean, one-way dependency flow, satisfying the Atlas persona's architectural requirements.
- **Decoupling**: The submodules are decoupled from the main orchestrator, making the codebase easier to reason about and mock in testing.

### Negative
- **Abstraction Overhead**: Introduces a single-use trait interface (`StatementAnalyzer`), which adds minor indirection when tracing code execution.
