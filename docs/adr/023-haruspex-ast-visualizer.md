# 023. Add Haruspex AST Visualizer Tool

Date: 2026-07-17

## Status

Accepted

## Context

Compiler developers for ΓΛΩΣΣΑ need to inspect the raw semantic tree structure to understand how expressions are nested and typed. Existing tools like Cartographer map architecture and Labyrinth traces control flow, but a tool to visualize the detailed semantic AST was missing.

## Decision

We have added the "Haruspex" (`ὁ Ἱεροσκόπος`) tool to the "Developer Experience (Nova)" toolset (`src/tools/haruspex.rs`). Haruspex inspects the `AnalyzedProgram` and translates it into a DOT graph for visualization with Graphviz.

## Consequences

* **Positive:** Compiler developers can visually inspect the detailed semantic AST using Graphviz.
* **Positive:** Follows the established pattern of implementing discrete developer tools within the `src/tools/` directory.
* **Negative:** Requires external tools (Graphviz) to fully utilize the generated DOT graphs.
