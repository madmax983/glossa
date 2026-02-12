# 9. Replace Oracle with Report and Highlight

Date: 2025-02-12

## Status

Accepted

## Context

The `Oracle` component was initially designed to provide explanation services, attempting to generate human-readable descriptions of the code's intent. However, its scope became ill-defined, overlapping with both error reporting and documentation generation. As the compiler matured, the need for distinct, specialized components became apparent:
1.  **Syntax Highlighting:** A purely presentational layer for the CLI (`highlight` command) to visualize the grammatical structure (Subject, Object, Verb) using ANSI colors.
2.  **Structured Reporting:** An analytical layer (`report` command) to provide statistics on code complexity, function counts, and other metrics, as well as structured error reporting.

The `Oracle` became a monolithic and obsolete abstraction that didn't fit well into the compilation pipeline.

## Decision

We will remove the `src/semantic/oracle.rs` component and replace its responsibilities with two new, focused modules:

1.  **Highlight (`src/highlight.rs`):**
    -   Responsible for semantic syntax highlighting.
    -   Consumes the AST directly from the Parser.
    -   Uses morphological analysis to color-code words based on their role (Case, Part of Speech).
    -   Outputs ANSI-colored text for the CLI.

2.  **Report (`src/report.rs`):**
    -   Responsible for generating compilation reports and program statistics.
    -   Consumes the `AnalyzedProgram` from the Semantic Analyzer.
    -   Calculates metrics like Statement Count, Expression Count, Max Depth, etc.
    -   Provides structured output for the `check` and `build` CLI commands.

The `Oracle` component will be removed from the codebase and the architecture documentation.

## Consequences

*   **Separation of Concerns:** The distinction between *visualizing* code structure (Highlight) and *analyzing* code metrics (Report) is now explicit.
*   **CLI Improvements:** The `glossa` CLI now supports dedicated `highlight` and `check` commands backed by these specialized modules.
*   **Reduced Complexity:** The Semantic Analysis phase is simplified by removing the `Oracle` dependency.
*   **Documentation Update:** The architecture diagrams must be updated to reflect these changes.
