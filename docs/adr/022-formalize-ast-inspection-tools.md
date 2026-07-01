# 22. Formalize AST Inspection Tools (Gnomon, Haruspex, Scholar)

Date: 2026-07-01
Status: Accepted

## Context

The compiler includes several tools for inspecting and interacting with the semantic AST:
- `Gnomon` (ὁ Γνώμων): Estimates Big-O time complexity by analyzing loop depth in the semantic AST.
- `Haruspex` (ὁ Ἱεροσκόπος): Translates the semantic AST into a DOT graph for Graphviz visualization.
- `Scholar` (ὁ Σχολαστικός): Parses programs and automatically generates Markdown API documentation (`doc.md`).

These tools exist within the `src/tools/` namespace and serve crucial functions for developer experience, but they are not currently formalized in the architectural maps. In accordance with the Codex philosophy of "Architectural Transparency," we must formally document these components.

## Decision

We officially recognize `Gnomon`, `Haruspex`, and `Scholar` as components within the "Developer Experience (Nova)" toolset.

## Consequences

- **Architectural Transparency**: These tools are now explicitly modeled in the System Architecture C4 diagrams.
- **Clarity of Purpose**: The tools' roles within the Nova toolset are documented and visible.
