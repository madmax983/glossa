# 022. Add Haruspex AST Visualizer

Date: 2026-05-09

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler translates source code into a complex semantic tree structure (`AnalyzedProgram`). While tools like the Cartographer map the architecture and the Labyrinth traces control flow, compiler developers need a way to inspect the raw semantic tree directly. Understanding exactly how expressions are nested and typed is crucial for debugging and improving the compiler.

## Decision

We have added "The Haruspex" (`ὁ Ἱεροσκόπος`) to the compiler toolset (`src/tools/haruspex.rs`). This tool inspects the semantic AST (`AnalyzedProgram`) of a ΓΛΩΣΣΑ program and translates it into a DOT graph for visualization with Graphviz.

## Consequences

- **Developer Experience:** Compiler developers can visually inspect the exact shape of the semantic AST, making it easier to debug parser and semantic analysis issues.
- **Dependency:** Introduces a dependency on Graphviz/DOT format for the visual output.
