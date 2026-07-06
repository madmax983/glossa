# 022. Add Haruspex AST Visualizer Tool

Date: 2026-05-10

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler developers need a way to inspect the raw semantic tree structure (`AnalyzedProgram`) to see exactly how expressions are nested and typed. While the Cartographer maps architecture and the Labyrinth traces control flow, there is a lack of tooling for generating DOT graphs of the semantic AST for visualization with Graphviz.

## Decision

We have added the "Haruspex" (`ὁ Ἱεροσκόπος`) tool to the "Developer Experience (Nova)" toolset (`src/tools/haruspex.rs`). Haruspex traverses the `AnalyzedProgram` and translates it into a DOT graph for visualization with Graphviz.

## Consequences

*   **Positive:** Compiler developers gain a powerful diagnostic tool for visually inspecting the semantic AST layout.
*   **Positive:** It acts as a consumer of the `AnalyzedProgram`, similar to other tools.
*   **Negative:** Adds a new tool that must be maintained as part of the `glossa` toolset.
