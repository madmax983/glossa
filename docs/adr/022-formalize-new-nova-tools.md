# 022. Formalize Gnomon, Haruspex, and Scholar Tools

Date: 2026-07-24

## Status

Proposed

## Context

As part of the continuous evolution of the "Developer Experience (Nova)" toolset, three new tools were introduced into the ΓΛΩΣΣΑ compiler:
1. **The Gnomon (`src/tools/gnomon.rs`)**: A Big-O complexity estimator that analyzes loop depths in the semantic AST.
2. **The Haruspex (`src/tools/haruspex.rs`)**: A Graphviz AST visualizer that translates the semantic AST into DOT graphs for deep structural inspection.
3. **The Scholar (`src/tools/scholar.rs`)**: An API documentation generator that parses a ΓΛΩΣΣΑ program and automatically generates comprehensive Markdown API documentation (`doc.md`) by extracting type definitions (Structs), traits (Interfaces), and verbs (Functions).

These tools are fully functional but are missing formal architectural documentation and representation in our architecture diagrams. This violates the project's principle of Architectural Transparency.

## Decision

We formally recognize "The Gnomon", "The Haruspex", and "The Scholar" as components within the "Developer Experience (Nova)" container (`src/tools/`).

- **The Gnomon** is defined as the Big-O Complexity Estimator tool.
- **The Haruspex** is defined as the Graphviz AST Visualizer tool.
- **The Scholar** is defined as the API Doc Generator tool.

They will be integrated into the architecture diagram (`docs/architecture.md`) alongside other Nova tools, properly showcasing their dependency on the `AnalyzedProgram` produced by the `Semantic Analyzer`.

## Consequences

- **Positive:** The roles of the new tools are explicitly documented and visualized, maintaining the "no implicit decisions" rule.
- **Positive:** Developer experience is enhanced by making the capabilities of these tools visible and understandable.
- **Negative:** The architecture diagram becomes slightly more dense, and these tools must be maintained as the semantic model evolves.
