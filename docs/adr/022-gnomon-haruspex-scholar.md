# 022. Add Gnomon, Haruspex, and Scholar to Developer Experience Tools

Date: 2026-06-26

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler's "Developer Experience (Nova)" toolset provides various utilities to help developers interact with and understand their programs. However, there were gaps in analyzing time complexity, deeply inspecting the AST, and generating API documentation. We needed dedicated tools to handle these aspects to improve the overall developer experience.

## Decision

We have added three new tools to the "Developer Experience (Nova)" toolset:
1.  **The Gnomon (`ὁ Γνώμων`):** Located at `src/tools/gnomon.rs`, this tool estimates the Big-O time complexity of a program by statically analyzing loop depth in the semantic AST.
2.  **The Haruspex (`ὁ Ἱεροσκόπος`):** Located at `src/tools/haruspex.rs`, this tool translates the semantic AST into a Graphviz DOT diagram, allowing compiler developers to inspect the raw semantic tree structure visually.
3.  **The Scholar (`ὁ Σχολαστικός`):** Located at `src/tools/scholar.rs`, this tool parses a program and automatically generates comprehensive Markdown API documentation for its structs, traits, and functions.

## Consequences

*   **Positive:** Developers have access to complexity estimation, deep AST visualization, and automatic documentation generation.
*   **Positive:** Continues the pattern of building modular, specialized tools that consume the `AnalyzedProgram` from the Semantic Analyzer.
*   **Negative:** Adds maintenance overhead for three new tools in the compiler toolset.
