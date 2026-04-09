# 17. Add Labyrinth Tool for Control Flow Graph Visualization

Date: 2026-04-08
Status: Proposed

## Context

"Architectural Transparency" is a core value of the ΓΛΩΣΣΑ project. As Glossa programs grow and incorporate complex logic (loops, conditionals, function calls), understanding the control flow and structure of the execution paths becomes more difficult. While we have tools like the Narrator and Mosaic for observing semantics and assembly, there was no direct way to visualize the flow of execution visually.

To solve this, the "Labyrinth" tool was introduced into `src/tools/labyrinth.rs`. This tool is capable of taking an analyzed Glossa program and emitting a Mermaid.js flowchart mapping out its control flow graph.

Because this represents a new high-level developer experience component that depends on the core semantic analysis phase, we must record its existence and architectural boundaries to maintain transparency.

## Decision

We formally recognize the "Labyrinth" tool as a component within the "Developer Experience (Nova)" toolset.
We will add `labyrinth` to the system architecture diagram as a Container within the tools boundary, which consumes the `AnalyzedProgram` from the Semantic Analyzer.

## Consequences

*   **Positive:** Provides a visual aid for tracing control flow in ΓΛΩΣΣΑ programs, improving the developer and educational experience.
*   **Positive:** Fulfills the "Architectural Transparency" requirement by allowing the inner logic of complex branches to be mapped out dynamically.
*   **Negative:** Adds another tool to the `tools` ecosystem that must be maintained alongside changes to the abstract syntax tree and the semantic model.
