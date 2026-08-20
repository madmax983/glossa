# 022. Add Haruspex, Scholar, and Gnomon to Developer Experience Tools

Date: 2026-08-16
Status: Proposed

## Context

As the ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset expands, three new experimental tools have been added to improve visibility into the semantic AST, automate documentation generation, and estimate code performance:

1.  **The Haruspex (`src/tools/haruspex.rs`)**: An experimental tool that exports the semantic AST (`AnalyzedProgram`) into a DOT graph for visualization with Graphviz.
2.  **The Scholar (`src/tools/scholar.rs`)**: An API documentation generator that parses the AST to document structures like structs.
3.  **The Gnomon (`src/tools/gnomon.rs`)**: A Big-O complexity estimator that statically analyzes loop depth in the semantic AST.

Currently, these tools are missing formal architectural representation, which violates the Codex principles of Architectural Transparency. We must explicitly define their roles and boundaries within the system context.

## Decision

We formally recognize "The Haruspex", "The Scholar", and "The Gnomon" as components within the "Developer Experience (Nova)" container (`src/tools/`).

These tools will be integrated into the architecture diagram (`docs/architecture.md`) alongside other Nova tools, and each will be modeled as consuming an `AnalyzedProgram` from the Semantic Analyzer.

## Consequences

### Positive
- **Architectural Clarity**: The roles of the new modules are explicitly documented and mapped, eliminating implicit decisions.
- **Enhanced Visibility**: Users and contributors can more easily discover the capabilities of the compiler to visualize ASTs, generate documentation, and estimate algorithm complexity.

### Negative
- **Maintenance Surface**: These tools must be maintained as the semantic model (`AnalyzedProgram`) evolves. Breakages in the AST will require updates to these exporters to ensure accurate output.
