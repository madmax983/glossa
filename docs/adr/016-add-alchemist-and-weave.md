# 16. Add Alchemist and Weave to Developer Experience Tools

Date: 2024-11-15
Status: Proposed

## Context

As the ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset expands, two new experimental exporter tools have been developed to enhance the language's interoperability and educational value:

1.  **The Alchemist (`src/tools/alchemist.rs`)**: An experimental transpiler that converts ΓΛΩΣΣΑ programs into Python scripts. This provides a dynamic, scripting-oriented target that complements the primary Rust codegen backend, proving the backend-independence of the semantic analysis phase.
2.  **Weave (`src/tools/weave.rs`)**: An exporter that generates a "Rosetta Stone" Markdown document. It combines the original Greek source code, the detailed semantic assembly logic, and the final generated Rust code into a single, structured view. This is designed to serve as a powerful educational and documentation tool.

Currently, these tools exist in the codebase but are missing formal architectural representation, which violates the Codex principles of Architectural Transparency. We must explicitly define their roles and boundaries within the system context.

## Decision

We formally recognize "The Alchemist" and "Weave" as components within the "Developer Experience (Nova)" container (`src/tools/`).

- We define **The Alchemist** as the Python Exporter / Transpiler tool.
- We define **Weave** as the Markdown "Rosetta Stone" Exporter tool.

Both tools will be integrated into the architecture diagram as containers that depend on the output of the `Semantic Analyzer`.

## Consequences

### Positive
- **Architectural Clarity**: The roles of the new `alchemist` and `weave` modules are explicitly documented and visualized, maintaining the "no implicit decisions" rule.
- **Demonstrated Modularity**: Recognizing The Alchemist highlights the decoupling of the semantic phase from the Rust-specific codegen phase.
- **Enhanced Documentation**: The "Rosetta Stone" format produced by Weave explicitly supports the project's educational goals.

### Negative
- **Maintenance Surface**: The compiler now has two new tools that must be maintained as the semantic model (`AnalyzedProgram`) evolves. Breakages in the AST or HIR will require updates to both exporters.
