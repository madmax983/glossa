# 022. Add Haruspex AST Visualizer Tool

Date: 2026-05-16

## Status

Proposed

## Context

While the Cartographer maps architecture and the Labyrinth traces control flow, compiler developers need a way to inspect the raw semantic tree structure (`AnalyzedProgram`), seeing exactly how expressions are nested and typed. Without such a tool, debugging the AST requires manual inspection or text-based dumps.

## Decision

We have added "Haruspex" (`ὁ Ἱεροσκόπος`) to the Developer Experience toolset in `src/tools/haruspex.rs`. Haruspex generates a Graphviz DOT representation of the analyzed semantic Abstract Syntax Tree (AST), allowing developers to visually inspect the internal compiler representations.

## Consequences

*   **Positive:** Developers can easily visualize the AST with Graphviz, significantly improving debugging and understanding of the compiler's semantic phase.
*   **Positive:** Continues the pattern of isolated Developer Experience tools in `src/tools/`.
*   **Negative:** Adds maintenance overhead; the DOT generator must be kept in sync with changes to `AnalyzedExpr` and `AnalyzedStatement` structures.
