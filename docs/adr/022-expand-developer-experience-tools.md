# 022. Expand Developer Experience Tools

Date: 2026-08-02

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler's semantic phase produces a complex Abstract Syntax Tree (`AnalyzedProgram`). While we have tools for executing, generating code, and visualizing the control flow, we lack specialized tools for inspecting the raw structural tree of the AST, statically estimating the time complexity of the program, and generating human-readable API documentation from the code itself. These gaps make it difficult for developers to deeply understand the program's structure, predict its performance, and share its API without manual documentation efforts.

## Decision

We have added three new tools to the Developer Experience (Nova) toolset:
1. **Haruspex** (`ὁ Ἱεροσκόπος`): A tool that inspects the semantic AST and translates it into a DOT graph for visualization with Graphviz, allowing developers to see exactly how expressions are nested and typed.
2. **Gnomon** (`ὁ Γνώμων`): A Big-O Complexity Estimator that statically analyzes loop depth in the semantic AST to estimate the program's time complexity.
3. **Scholar** (`ὁ Σχολαστικός`): An API Doc Generator that parses the program to automatically generate comprehensive Markdown API documentation for defined structures, traits, and functions.

## Consequences

*   **Positive:** Developers gain deeper visibility into the AST structure, can easily estimate execution time complexity, and automatically generate API documentation, significantly improving the developer experience.
*   **Positive:** Adheres to the established pattern of creating discrete, single-responsibility developer tools in the `src/tools/` directory.
*   **Negative:** Adds three new tools that must be maintained and updated as the compiler's semantic model evolves.
