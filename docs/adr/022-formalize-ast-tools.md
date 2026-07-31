# 22. Formalize AST Inspection Tools

Date: 2025-02-12
Status: Accepted

## Context

Recent refactoring by Atlas (commit `e27cd2c`) moved three internal diagnostic and documentation tools—`haruspex`, `gnomon`, and `scholar`—into the `src/tools/` directory to consolidate all auxiliary utilities.

- **The Haruspex**: A Graphviz AST Visualizer for inspecting the raw semantic tree structure.
- **The Gnomon**: A Big-O Complexity Estimator based on loop depth analysis.
- **The Scholar**: An API Documentation Generator for creating Markdown reference guides from the codebase.

While these tools were correctly placed into the Developer Experience (`tools`) namespace in the codebase, this structural change was missing formal architectural representation. This violates the core principle of Architectural Transparency, leaving these tools undocumented in our System Context maps.

## Decision

We have explicitly recorded the addition of `haruspex`, `gnomon`, and `scholar` to the Developer Experience toolset. We have updated the C4 Container diagram in `docs/architecture.md` to map their boundaries and explicitly illustrate their relationships with the `semantic` module, as they all act upon the `AnalyzedProgram` AST.

## Consequences

- **Architectural Transparency**: The architectural maps now accurately reflect the full suite of developer tools available in the codebase.
- **System Comprehension**: It is now clear that these tools sit at the same level as other Nova tools (like Mentor and Labyrinth) and rely on the outputs of the Semantic Analyzer.
- **Living Maps**: By recording this decision and updating the diagram, we ensure our documentation remains a living representation of our code structure rather than falling out of sync.
