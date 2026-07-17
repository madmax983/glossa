# 024. Add Scholar API Doc Generator Tool

Date: 2026-07-17

## Status

Accepted

## Context

To bridge the gap between raw semantic analysis and human-readable references, ΓΛΩΣΣΑ needed a way to automatically generate documentation for types, traits, and functions defined in Ancient Greek source files. Without it, developers would have to read the source files to understand the API.

## Decision

We have added the "Scholar" (`ὁ Σχολαστικός`) tool to the "Developer Experience (Nova)" toolset (`src/tools/scholar.rs`). Scholar parses a target program and extracts struct definitions, traits, and functions, formatting them into standardized GitHub-flavored Markdown.

## Consequences

* **Positive:** Automates API documentation generation, greatly improving developer experience and library usability.
* **Positive:** Seamlessly integrates with the existing toolset as a consumer of the `AnalyzedProgram`.
* **Negative:** Requires updates whenever the semantic representation of types, traits, or functions changes.
