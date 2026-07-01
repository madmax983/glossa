# 023. Haruspex AST Visualizer

Date: 2026-06-30
Status: Accepted

## Context

Compiler developers and language researchers often need to inspect the raw semantic tree structure to understand exactly how expressions are nested and typed. Text-based outputs or existing visualizers (like Cartographer for architecture) did not provide a granular, node-level view of the Semantic AST.

## Decision

We implemented the "Haruspex" (ὁ Ἱεροσκόπος) tool in `src/tools/haruspex.rs`. It inspects the `AnalyzedProgram` of a ΓΛΩΣΣΑ program and translates it into a DOT graph for visualization with Graphviz.

## Consequences

- **Positive:** Provides deep, granular visibility into the compiler's semantic analysis phase.
- **Positive:** Aids in debugging complex nested expressions and type resolutions.
- **Negative:** Requires external tools (Graphviz) to render the final visual output from the DOT graph.
