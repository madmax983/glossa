# 022. Add Scholar Tool to Developer Experience Tools

Date: 2026-05-10

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler needed a way to bridge the gap between raw semantic analysis and human-readable API references. Without documentation, users cannot easily understand defined structures (`εἴδη`), traits (`χαρακτῆρες`), and functions (`ἔργα`).

## Decision

We have added the "Scholar" (`ὁ Σχολαστικός`) tool to the "Developer Experience (Nova)" toolset (`src/tools/scholar.rs`). It acts as an API Doc Generator that parses a program, extracts semantic definitions, and outputs a clean, GitHub-flavored Markdown file.

## Consequences

*   **Positive:** Developers can generate accurate API documentation effortlessly, proving the power of a centralized semantic model.
*   **Positive:** Integrates seamlessly into the existing semantic AST tooling.
*   **Negative:** Adds a new tool that must be maintained as part of the `glossa` toolset.
