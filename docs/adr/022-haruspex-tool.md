# 22. Add Haruspex to Developer Experience Tools

Date: 2026-05-10
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler includes the "Haruspex" (ὁ Ἱεροσκόπος) tool (`src/tools/haruspex.rs`).
While the Cartographer maps architecture and the Labyrinth traces control flow, the Haruspex allows compiler developers to inspect the raw semantic tree structure.
It translates the semantic AST (`AnalyzedProgram`) of a ΓΛΩΣΣΑ program into a DOT graph for visualization with Graphviz.

This tool exists but has not been documented with an Architecture Decision Record or in the architecture diagrams, which violates the Codex principles of Architectural Transparency.

## Decision

We formally recognize the **Haruspex** tool as a component within the "Developer Experience (Nova)" container (`src/tools/`).

Haruspex is defined as the Graphviz AST Visualizer that takes an `AnalyzedProgram` from the Semantic Analyzer and generates a DOT graph. It will be integrated into the architecture diagram (`docs/architecture.md`) alongside other Nova tools.

## Consequences

*   **Positive:** The internal structure of the `AnalyzedProgram` can be easily inspected, allowing compiler developers to see exactly how expressions are nested and typed.
*   **Positive:** Architectural Clarity: The role and existence of the Haruspex tool are explicitly documented and mapped, eliminating implicit decisions.
*   **Negative:** As a recognized tool, Haruspex must be maintained to keep pace with changes to the semantic model (`AnalyzedProgram`), to ensure it can correctly traverse and emit the DOT graph for all expressions and statements.
