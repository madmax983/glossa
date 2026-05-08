# 22. Haruspex - Graphviz AST Visualizer

Date: 2026-05-08
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler includes several tools in the "Developer Experience (Nova)" toolset. A new tool, "Haruspex" (`src/tools/haruspex.rs`), has been introduced to inspect the semantic AST (`AnalyzedProgram`) and translate it into a Graphviz DOT representation.

While the Cartographer maps architecture and the Labyrinth traces control flow, the Haruspex allows compiler developers to inspect the raw semantic tree structure.

Recently, the module was refactored by the "Razor" persona to flatten the `DotGenerator` struct into simple functions, improving code essentialism and reducing unnecessary abstraction. However, this tool has lacked a formal architectural record.

## Decision

We formally recognize "Haruspex" as a core component within the "Developer Experience (Nova)" toolset container. It functions as the Graphviz AST Visualizer. It will be added to the C4 Container architecture diagram as a container.

## Consequences

### Positive
- **Architectural Clarity**: The role of the `haruspex` module is explicitly documented and visualized, maintaining the "no implicit decisions" rule of Codex.
- **Improved Tooling Visibility**: Compiler developers have clear visibility that this developer tool exists for debugging AST issues visually.

### Negative
- **Maintenance Surface**: The compiler has another developer tool that must be maintained as the semantic AST model (`AnalyzedProgram` / `AnalyzedStatement` / `AnalyzedExpr`) evolves. The Graphviz DOT generator logic must stay in sync with tree structure changes.
