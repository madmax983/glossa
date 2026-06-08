# 022. Introduce DX Tools: Gnomon, Haruspex, and Scholar

Date: 2026-05-15
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler provides semantic analysis of ancient Greek source code. As the language matures, developers require more visibility into the compiler's behavior and the ability to generate standard artifacts. Previously, understanding the control flow or time complexity of an algorithm required manual tracing or running external tools. Furthermore, generating documentation required manually inspecting the codebase, as no automated API generator existed for the `.γλ` format.

## Decision

We have introduced three new Developer Experience (DX) tools to the `src/tools/` directory, operating on the semantic AST (`AnalyzedProgram`):
- **Gnomon (`gnomon.rs`)**: A Big-O Complexity Estimator that analyzes loop depth and control flow to estimate the time complexity of the program.
- **Haruspex (`haruspex.rs`)**: A Graphviz AST Visualizer that translates the semantic tree into DOT format for detailed inspection.
- **Scholar (`scholar.rs`)**: An API documentation generator that extracts structural definitions (`εἴδη`, `χαρακτῆρες`, `ἔργα`) and produces Markdown documentation.

## Consequences

- **Enhanced Developer Experience**: Developers can now automatically estimate performance, visually debug the AST, and generate library documentation, significantly improving the development lifecycle.
- **Boundary Expansion**: The "Developer Experience (Nova)" tool boundary in the architecture grows, adding more dependencies on the core semantic engine.
- **Maintenance**: These tools must be maintained and updated as the `AnalyzedProgram` structure evolves, requiring corresponding tests and architectural map updates.
