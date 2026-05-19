# 022. Diagnostic and Documentation Tools

Date: 2026-05-19

## Status

Proposed

## Context

As the ΓΛΩΣΣΑ compiler's feature set and user base expand, there is an increasing need to provide developers with insights into their code beyond basic execution and parsing. Specifically, understanding the time complexity, generating readable documentation from the raw Ancient Greek source code, and visualizing the abstract syntax tree are key steps to improving the developer experience. The modules `gnomon.rs`, `haruspex.rs`, and `scholar.rs` have been introduced to the tool suite to fulfill these needs, but their addition lacks formal architectural documentation.

## Decision

We formally recognize and document the addition of the following modules to the `src/tools/` ecosystem, treating them as part of the core "Nova" Developer Experience:

1.  **The Gnomon (`src/tools/gnomon.rs`)**: A tool that estimates the Big-O time complexity of a program by statically analyzing loop depth in the semantic AST.
2.  **The Haruspex (`src/tools/haruspex.rs`)**: A visualization tool that inspects the semantic AST and translates it into a DOT graph for rendering with Graphviz. This enables developers to see exactly how expressions are nested and typed.
3.  **The Scholar (`src/tools/scholar.rs`)**: An API documentation generator that parses `.γλ` files and distills type definitions, traits, and functions into GitHub-flavored Markdown. This makes APIs accessible without needing to read the raw Ancient Greek source files.

## Consequences

*   **Positive:** The compiler now provides enhanced tooling for performance estimation (Gnomon), visual debugging of the AST (Haruspex), and library documentation generation (Scholar).
*   **Positive:** The roles of these tools are explicitly recorded, ensuring clarity in their purpose within the wider compiler architecture.
*   **Negative:** Added complexity to the `tools` module that must be maintained as the core AST and language features evolve.
