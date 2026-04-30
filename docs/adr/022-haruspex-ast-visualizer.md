# 022. Add Haruspex AST Visualizer

Date: 2026-04-30

## Status

Accepted

## Context

While the Labyrinth tool visualizes the control flow graph and the Cartographer maps architecture, compiler developers needed a way to inspect the raw semantic tree structure. It is often necessary to see exactly how expressions are nested and typed in the `AnalyzedProgram` to debug semantic analysis and code generation.

## Decision

We have added the "Haruspex" (ὁ Ἱεροσκόπος) tool to the Developer Experience toolset (`src/tools/haruspex.rs`).
The Haruspex inspects the semantic AST (`AnalyzedProgram`) of a ΓΛΩΣΣΑ program and translates it into a DOT graph for visualization with Graphviz.

## Consequences

*   **Positive:** Compiler developers can visually inspect the exact structure of the HIR/AST, improving debugging capabilities.
*   **Positive:** The tool integrates cleanly into the existing pipeline by consuming the `AnalyzedProgram`.
*   **Negative:** Adds another tool to the `tools` module that needs maintenance.
