# 023. Add Haruspex Tool to Developer Experience Tools

Date: 2026-07-08

## Status

Proposed

## Context

While tools like Cartographer and Labyrinth provide high-level and control-flow insights respectively, compiler engineers lack a way to granularly inspect the raw structure of the semantic Abstract Syntax Tree (AST). Visualizing how expressions are nested and typed is critical for debugging semantic analysis in ΓΛΩΣΣΑ.

## Decision

We have added the "Haruspex" (`ὁ Ἱεροσκόπος`) tool in `src/tools/haruspex.rs` to the "Developer Experience (Nova)" toolset. Haruspex traverses the `AnalyzedProgram` from the Semantic Analyzer and generates a DOT graph that visually maps the nested expressions and type bindings, suitable for rendering with Graphviz.

## Consequences

*   **Positive:** Developers can visually inspect and debug the semantic AST.
*   **Positive:** Enhances overall observability of the internal AST state.
*   **Negative:** Adds a new tool that must be maintained as part of the `glossa` toolset.
