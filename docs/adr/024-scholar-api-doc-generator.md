# 024. Add Scholar API Doc Generator Tool

Date: 2026-07-28

## Status

Proposed

## Context

In the modern software era, undocumented libraries hinder adoption and usability. For ΓΛΩΣΣΑ programs, there is a missing link between raw semantic analysis (the AST) and human-readable references. Developers need a way to understand an API's shape (structs, traits, and functions) without being forced to read raw Ancient Greek source files. We need an automated way to distill these constructs into accessible, standardized documentation.

## Decision

We have added "Scholar" (`Σχολαστικός`) to the "Developer Experience (Nova)" toolset (`src/tools/scholar.rs`). The Scholar tool parses a `.γλ` program, extracts type definitions (Structs), traits (Interfaces), and verbs (Functions), and formats them into standardized, GitHub-flavored Markdown API documentation (`doc.md`).

## Consequences

* **Positive:** Bridges the gap between semantic analysis and human-readable references, improving library documentation and usability for developers.
* **Positive:** Automates the creation of standardized API documentation, reducing manual effort.
* **Negative:** The tool must be maintained alongside the core parser and semantic analyzer to ensure it correctly extracts all new API constructs added to the language.
