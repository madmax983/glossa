# 022. Add Nova AST Tools (Gnomon, Haruspex, Scholar)

Date: 2026-05-05

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset (`src/tools/`) has been expanding to include tools that consume the semantic AST (`AnalyzedProgram`). Three such tools are missing formal architectural representation, violating the Codex principles of Architectural Transparency:

1.  **Gnomon (`src/tools/gnomon.rs`)**: A Big-O complexity estimator that statically analyzes loop depth in the semantic AST.
2.  **Haruspex (`src/tools/haruspex.rs`)**: A tool that visualizes the AST as a GraphViz DOT graph, providing insights into the structure of programs.
3.  **Scholar (`src/tools/scholar.rs`)**: An API documentation generator that parses programs and generates Markdown documentation for types, traits, and functions.

## Decision

We formally recognize "Gnomon", "Haruspex", and "Scholar" as components within the "Developer Experience (Nova)" container (`src/tools/`).

- **Gnomon** is the Big-O time complexity estimator.
- **Haruspex** is the GraphViz AST visualizer.
- **Scholar** is the API doc generator.

All three tools will be integrated into the architecture diagram (`docs/architecture.md`) alongside other Nova tools, showing they consume the `AnalyzedProgram` from the Semantic Analyzer.

## Consequences

- **Architectural Clarity**: Explicitly documenting these tools maintains the "no implicit decisions" rule.
- **Enhanced Visibility**: Users and contributors can more easily discover and understand the purpose of the new diagnostic and documentation features.
- **Maintenance Surface**: These tools must be maintained as part of the Nova toolset and may require updates if the semantic model (`AnalyzedProgram`) evolves.
