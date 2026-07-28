# 023. Add Haruspex AST Visualizer Tool

Date: 2026-07-28

## Status

Proposed

## Context

While the ΓΛΩΣΣΑ compiler provides tools like the Cartographer to map high-level architecture and the Labyrinth to trace control flow, compiler developers often struggle to visualize the exact structure of the internal semantic Abstract Syntax Tree (`AnalyzedProgram`). There is a need for a tool that allows developers to inspect the raw semantic tree structure to see exactly how expressions are nested and typed, aiding in debugging and compiler development.

## Decision

We have added "Haruspex" (`Ἱεροσκόπος`) to the "Developer Experience (Nova)" toolset (`src/tools/haruspex.rs`). Haruspex inspects the semantic AST of a ΓΛΩΣΣΑ program and translates it into a DOT graph format. This allows the structural layout of the AST to be visualized using Graphviz, exposing the internal semantic representation clearly.

## Consequences

* **Positive:** Compiler developers gain a powerful diagnostic tool to visualize the exact semantic AST, making it easier to debug parser or semantic analysis issues.
* **Positive:** Extends the existing architectural pattern of tools consuming the `AnalyzedProgram` without interfering with core compilation.
* **Negative:** The generated DOT graphs can become very large and difficult to read for complex programs.
