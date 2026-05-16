# 022. Add Haruspex, Gnomon, and Scholar to Developer Experience Tools

Date: 2026-05-16

## Status

Proposed

## Context

As the ΓΛΩΣΣΑ compiler evolves, the need for enhanced diagnostic and analytical tools has grown. Developers require ways to inspect the raw semantic tree structure for debugging, estimate the time complexity of programs statically, and automatically generate comprehensive API documentation from source code. Specifically:
- A tool is needed to translate the semantic AST (`AnalyzedProgram`) into a visual DOT graph for Graphviz to help compiler developers understand how expressions are nested and typed.
- A tool is needed to estimate the Big-O time complexity by statically analyzing loop depth in the semantic AST.
- A tool is needed to bridge the gap between raw semantic analysis and human-readable references by automatically generating Markdown API documentation from type definitions, traits, and functions.

## Decision

We have added three new tools to the "Developer Experience (Nova)" toolset (`src/tools/`):
- **Haruspex (`src/tools/haruspex.rs`)**: Inspects the semantic AST and translates it into a DOT graph for visualization.
- **Gnomon (`src/tools/gnomon.rs`)**: Estimates the Big-O time complexity by analyzing loop depth in the AST.
- **Scholar (`src/tools/scholar.rs`)**: Parses a program and automatically generates comprehensive Markdown API documentation.

These tools are integrated into the architecture as consumers of the `AnalyzedProgram` from the Semantic Analyzer.

## Consequences

* **Positive:** Developers have powerful new ways to visualize ASTs, estimate performance, and auto-generate documentation, vastly improving the developer experience.
* **Positive:** The tools adhere to the established architectural pipeline and boundary, keeping core compiler logic separate.
* **Negative:** The addition of these tools increases the overall maintenance burden of the codebase. Breakages in the AST or HIR may require updates to all three tools.
