# 023. Add Haruspex (ὁ Ἱεροσκόπος) Graphviz AST Visualizer

Date: 2026-08-19
Status: Proposed

## Context
Compiler development requires deep inspection of intermediate representations. While existing tools map high-level architecture (Cartographer) or control flow (Labyrinth), there was a missing capability to visualize the raw, nested structure and types of the semantic Abstract Syntax Tree (`AnalyzedProgram`). The "Haruspex" tool (`src/tools/haruspex.rs`) was built to translate the AST into a DOT graph for Graphviz, but this structural decision remains undocumented.

## Decision
We formally add the **Haruspex** tool to the architectural records as a component of the "Developer Experience (Nova)" container (`src/tools/`).
Haruspex is defined as the raw semantic AST visualization tool, converting `AnalyzedProgram` structures into Graphviz DOT format to expose precise nesting and type information.

## Consequences
- **Positive:** Enhanced debuggability for compiler developers who can now visualize the exact shape of the semantic tree.
- **Negative:** Haruspex is tightly coupled to the structure of `AnalyzedExpr` and `AnalyzedStatement`. Any changes to the semantic model require immediate updates to the DOT generation logic.
