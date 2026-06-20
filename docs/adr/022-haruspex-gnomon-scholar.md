# 22. Add Haruspex, Gnomon, and Scholar to Developer Experience Tools

Date: 2026-06-20
Status: Proposed

## Context

As the ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset expands, three new tools have been developed to enhance the language's introspectability and analytical power:

1.  **The Haruspex (`src/tools/haruspex.rs`)**: An experimental tool that exports the semantic AST directly to a Graphviz DOT diagram, allowing for visual inspection of exactly how the Assembler routed cases and typed nodes.
2.  **The Gnomon (`src/tools/gnomon.rs`)**: An experimental tool that estimates the Big-O time complexity of a ΓΛΩΣΣΑ program by statically analyzing loop depth in the semantic AST.
3.  **The Scholar (`src/tools/scholar.rs`)**: An experimental Markdown documentation generator that uses the compiler's semantic phase to extract and format types, traits, and functions.

Currently, these tools exist in the codebase but are missing formal architectural representation, which violates the Codex principles of Architectural Transparency. We must explicitly define their roles and boundaries within the system context.

## Decision

We formally recognize "The Haruspex", "The Gnomon", and "The Scholar" as components within the "Developer Experience (Nova)" container (`src/tools/`).

- We define **The Haruspex** as the Graphviz AST Visualizer.
- We define **The Gnomon** as the Big-O Complexity Estimator.
- We define **The Scholar** as the API Doc Generator.

These tools will be integrated into the architecture diagram as containers that depend on the output of the `Semantic Analyzer`.

## Consequences

### Positive
- **Architectural Clarity**: The roles of the new `haruspex`, `gnomon`, and `scholar` modules are explicitly documented and visualized, maintaining the "no implicit decisions" rule.
- **Enhanced Analysis and Documentation**: Statically analyzing the AST for complexity (Gnomon), generating visual representations (Haruspex), and extracting API boundaries (Scholar) drastically improves the tools available to ΓΛΩΣΣΑ developers.

### Negative
- **Maintenance Surface**: The compiler now has three new tools that must be maintained as the semantic model (`AnalyzedProgram`) evolves. Breakages in the AST or HIR will require updates to these exporters and analyzers.
