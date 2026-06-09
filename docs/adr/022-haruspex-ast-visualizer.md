# 022. Add Haruspex AST Visualizer Tool

Date: 2026-05-09

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler translates source code into an internal Semantic Abstract Syntax Tree (AST), defined by `AnalyzedProgram`. As the language grows more complex, visualizing the deeply nested structures and typed expressions directly from raw compiler logs or generic debug outputs has become challenging for compiler developers.
There is a need for a targeted visualization tool to represent the raw semantic tree structure graphically to help engineers inspect how statements and nested expressions are represented internally.

## Decision

We have introduced the "Haruspex" (ὁ Ἱεροσκόπος) tool to the Developer Experience (Nova) suite (`src/tools/haruspex.rs`).
The Haruspex inspects the `AnalyzedProgram` from the Semantic Analyzer and translates the semantic AST directly into a Graphviz DOT representation. The output can be piped to standard graph rendering tools (like `dot`) to produce structural node-and-edge graphs of the program's hierarchy.

## Consequences

* **Positive:** Compiler developers have a powerful diagnostic tool to visualize and debug the semantic AST directly.
* **Positive:** Continues our goal of Architectural Transparency by visually exposing internal compiler state.
* **Negative:** Introduces an additional maintenance surface. The Haruspex DOT generation logic must be updated whenever new variants or fields are added to `AnalyzedStatement` or `AnalyzedExpr` to ensure accurate visualization.
