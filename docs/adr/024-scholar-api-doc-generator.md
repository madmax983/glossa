# 024. Add Scholar API Doc Generator

Date: 2026-05-09

## Status

Proposed

## Context

In modern software development, API documentation is critical for adoption and usability. Currently, understanding the shape of an API in ΓΛΩΣΣΑ requires reading the Ancient Greek source code. There is a need to extract type definitions (`εἴδη`), traits (`χαρακτῆρες`), and functions (`ἔργα`) into a clean, human-readable reference.

## Decision

We have added "The Scholar" (`ὁ Σχολαστικός`) to the compiler toolset (`src/tools/scholar.rs`). The Scholar parses a ΓΛΩΣΣΑ program (`AnalyzedProgram`) and automatically distills its structures, traits, and functions into comprehensive GitHub-flavored Markdown API documentation (`doc.md`).

## Consequences

- **Documentation Generation:** Automates the creation of standard Markdown API documentation, significantly improving the ecosystem's usability.
- **Maintainability:** Bridges the gap between the AST and human-readable references, freeing developers from writing redundant documentation manually.
