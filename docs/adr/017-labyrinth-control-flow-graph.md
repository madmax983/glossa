# 17. Add Labyrinth to Developer Experience Tools

Date: 2024-11-15
Status: Accepted

## Context

The ΓΛΩΣΣΑ compiler includes an experimental "Labyrinth" tool (`src/tools/labyrinth.rs`) as part of the "Developer Experience (Nova)" toolset. This tool is responsible for visualizing the control flow graph of a ΓΛΩΣΣΑ program as a Mermaid.js flowchart.

As programs grow in complexity, understanding the paths of execution (such as loops, conditionals, and function calls) becomes increasingly challenging. The Labyrinth tool addresses this by rendering these paths visible, aligning with the project's core value of "Architectural Transparency".

However, this tool is currently missing formal architectural documentation and representation in our architectural diagrams, which violates the Codex principles of Architectural Transparency.

## Decision

We formally recognize the **Labyrinth** tool as a component within the "Developer Experience (Nova)" container (`src/tools/`).

Labyrinth is defined as the control flow graph visualization tool that takes an `AnalyzedProgram` from the Semantic Analyzer and generates a Mermaid.js flowchart. It will be integrated into the architecture diagram (`docs/architecture.md`) alongside other Nova tools.

## Consequences

### Positive
- **Architectural Clarity**: The role and existence of the Labyrinth tool are explicitly documented and mapped, eliminating implicit decisions.
- **Enhanced Visibility**: Users and contributors can more easily discover and understand the purpose of the control flow visualization feature.
- **System Documentation**: The architecture diagram accurately reflects the current state of the compiler's toolset.

### Negative
- **Maintenance Surface**: As a recognized tool, Labyrinth must be maintained to keep pace with changes to the semantic model (`AnalyzedProgram`). Updates to the control flow logic in the AST or HIR will require corresponding updates to the Labyrinth tool to ensure accurate visualization.