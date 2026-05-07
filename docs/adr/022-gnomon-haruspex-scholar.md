# 22. Add Gnomon, Haruspex, and Scholar to Developer Experience Tools

Date: 2024-11-15
Status: Proposed

## Context

As the ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset expands, three new tools have been developed to enhance static analysis, visibility, and documentation generation:

1.  **The Gnomon (`src/tools/gnomon.rs`)**: A Big-O complexity estimator that statically analyzes loop depth in the semantic AST.
2.  **The Haruspex (`src/tools/haruspex.rs`)**: A Graphviz AST visualizer that translates the `AnalyzedProgram` into a DOT graph for visual inspection of the raw semantic tree structure.
3.  **The Scholar (`src/tools/scholar.rs`)**: An API documentation generator that distills defined structures, traits, and functions into GitHub-flavored Markdown.

Currently, these tools exist in the codebase but are missing formal architectural representation, which violates the Codex principles of Architectural Transparency. We must explicitly define their roles and boundaries within the system context.

## Decision

We formally recognize "The Gnomon", "The Haruspex", and "The Scholar" as components within the "Developer Experience (Nova)" container (`src/tools/`).

- We define **The Gnomon** as the Big-O Complexity Estimator tool.
- We define **The Haruspex** as the Graphviz AST Visualizer tool.
- We define **The Scholar** as the API Doc Generator tool.

All three tools will be integrated into the architecture diagram as containers that depend on the output of the `Semantic Analyzer` (`AnalyzedProgram`).

## Consequences

### Positive
- **Architectural Clarity**: The roles of the new `gnomon`, `haruspex`, and `scholar` modules are explicitly documented and visualized, eliminating implicit decisions.
- **Enhanced Visibility**: Users and contributors can more easily discover and understand the purpose of these developer experience tools.

### Negative
- **Maintenance Surface**: The compiler now has three new tools that must be maintained as the semantic model (`AnalyzedProgram`) evolves. Changes to the AST or HIR will require updates to these tools to ensure accurate analysis, visualization, and documentation.
