# 022. Add Scholar, Gnomon, and Haruspex to Developer Experience Tools

Date: 2026-08-15
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset (`src/tools/`) has been expanded with three new experimental tools to assist developers with documentation, performance estimation, and compiler development:

1.  **The Scholar (`src/tools/scholar.rs`)**: An API documentation generator that parses a ΓΛΩΣΣΑ program and generates comprehensive Markdown documentation (`doc.md`), detailing structs, traits, and functions.
2.  **The Gnomon (`src/tools/gnomon.rs`)**: A Big-O complexity estimator that statically analyzes loop depth in the semantic AST to estimate execution time complexity.
3.  **The Haruspex (`src/tools/haruspex.rs`)**: A Graphviz AST visualizer that translates the semantic AST into a DOT graph, helping developers inspect nested expressions and types.

Currently, these tools exist in the codebase but are missing formal architectural representation, which violates the Codex principles of Architectural Transparency. We must explicitly define their roles and boundaries within the system context.

## Decision

We formally recognize **Scholar**, **Gnomon**, and **Haruspex** as components within the "Developer Experience (Nova)" container (`src/tools/`).

- **Scholar**: API Documentation Generator.
- **Gnomon**: Big-O Complexity Estimator.
- **Haruspex**: Graphviz AST Visualizer.

All three tools consume the `AnalyzedProgram` from the Semantic Analyzer. They will be added to the architecture diagram (`docs/architecture.md`) alongside other Nova tools.

## Consequences

- **Positive**: The roles of the new tools are explicitly documented and mapped, eliminating implicit decisions.
- **Positive**: The architecture diagram accurately reflects the current state of the compiler's toolset.
- **Negative**: Adds three new tools that must be maintained as the semantic model (`AnalyzedProgram`) evolves. Breakages in the AST or HIR will require updates to these tools.
