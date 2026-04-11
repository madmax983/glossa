# 018. Add Auditor Tool

**Status:** Proposed

## Context

With the expansion of the Nova developer experience toolkit, we need a way to perform static analysis on user code to detect common code smells such as unused variables and mutable bindings that are never reassigned. The codebase currently lacks a dedicated linter for catching these semantic issues early in the compilation process.

## Decision

Introduce a new `Auditor` tool (`src/tools/auditor.rs`) to traverse the semantically analyzed Abstract Syntax Tree (`AnalyzedProgram`). The tool acts as an AST visitor that records variable declarations, mutations, and usages, issuing warnings when anomalies are detected.

## Consequences

- The Auditor provides immediate feedback on redundant code, improving code quality.
- This creates an additional traversal pass over the AST during the analysis phase.
- Further rules and code smell checks can be cleanly integrated into the visitor implementation within `src/tools/auditor.rs`.
