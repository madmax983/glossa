# 024. Add Scholar API Doc Generator Tool

Date: 2026-05-16

## Status

Proposed

## Context

In the modern software ecosystem, documented APIs are crucial. Without a dedicated documentation generator, developers must read raw Ancient Greek source code (`.γλ` files) to understand the shape of libraries, `εἴδη` (Structs), `χαρακτῆρες` (Traits), and `ἔργα` (Functions). We needed a bridge between the semantic AST and human-readable references.

## Decision

We have created the "Scholar" (`ὁ Σχολαστικός`) tool in `src/tools/scholar.rs`. Scholar acts as an API documentation generator that parses the target source file, extracts type definitions, traits, and functions from the semantic scope, and formats them into a standardized, GitHub-flavored Markdown file (`doc.md`).

## Consequences

*   **Positive:** Greatly improves the developer experience and library usability by automatically providing structured API documentation.
*   **Positive:** Saves developers from manually writing API boilerplate documents.
*   **Negative:** Adds maintenance complexity, as the tool must be updated whenever new structures or language features are introduced into the semantic models.
