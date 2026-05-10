# 022. Add Haruspex AST Visualizer Tool

Date: 2026-05-10

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler translates source code into a detailed semantic Abstract Syntax Tree (AST), known internally as the `AnalyzedProgram`. As the complexity of the compiler and the programs it processes increases, developers need a way to inspect this internal representation to understand how expressions are nested, typed, and structured by the semantic analyzer. While we have tools for control flow (Labyrinth) and architectural mapping (Cartographer), there was no dedicated utility for visualizing the raw semantic tree itself.

## Decision

We have added the "Haruspex" (`ὁ Ἱεροσκόπος`) tool to the "Developer Experience (Nova)" toolset (`src/tools/haruspex.rs`).
The Haruspex inspects the semantic AST (`AnalyzedProgram`) and translates it into a DOT graph format. This DOT output can then be rendered using Graphviz into visual diagrams (e.g., PNG or SVG), allowing developers to see the exact structure of the semantic tree.

## Consequences

* **Positive:** Developers and contributors have a powerful visual tool for debugging the parser and semantic analyzer, making the internal AST structures fully transparent.
* **Positive:** Aligns with the project's core value of "Architectural Transparency".
* **Negative:** The Haruspex adds to the maintenance burden of the `Nova` toolset, requiring updates whenever the underlying semantic AST structures (`AnalyzedProgram`, `AnalyzedStatement`, `AnalyzedExpr`) are modified.
