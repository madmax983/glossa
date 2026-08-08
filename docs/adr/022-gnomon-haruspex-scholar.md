# 022. Add Gnomon, Haruspex, and Scholar to Developer Experience Tools

Date: 2026-08-08

## Status

Proposed

## Context

As the ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset expands, three new experimental tools have been developed to enhance the language's analysis and documentation capabilities:

1.  **The Gnomon (`src/tools/gnomon.rs`)**: A Big-O Complexity Estimator that analyzes loop depth in the semantic AST to estimate the program's execution time complexity.
2.  **The Haruspex (`src/tools/haruspex.rs`)**: A Graphviz AST Visualizer that translates the semantic AST into a DOT graph for visualization.
3.  **The Scholar (`src/tools/scholar.rs`)**: An API Doc Generator that parses a ΓΛΩΣΣΑ program and generates comprehensive Markdown API documentation (`doc.md`).

Currently, these tools exist in the codebase but are missing formal architectural representation, which violates the Codex principles of Architectural Transparency. We must explicitly define their roles and boundaries within the system context.

## Decision

We formally recognize "The Gnomon", "The Haruspex", and "The Scholar" as components within the "Developer Experience (Nova)" container (`src/tools/`).

- We define **The Gnomon** as the Big-O Complexity Estimator tool.
- We define **The Haruspex** as the Graphviz AST Visualizer tool.
- We define **The Scholar** as the API Doc Generator tool.

These tools will be integrated into the architecture diagram as containers that depend on the output of the `Semantic Analyzer`.

## Consequences

- Improved Architectural Clarity: The roles of the new `gnomon`, `haruspex`, and `scholar` modules are explicitly documented and visualized, maintaining the "no implicit decisions" rule.
- Enhanced Analysis and Documentation: Recognizing these tools highlights the compiler's growing capabilities in static analysis, visualization, and automated documentation generation.
- Increased Maintenance Surface: The compiler now has three new tools that must be maintained as the semantic model (`AnalyzedProgram`) evolves. Breakages in the AST or HIR will require updates to these tools.
