# 024. Add Scholar Tool to Developer Experience Tools

Date: 2026-07-08

## Status

Proposed

## Context

The ΓΛΩΣΣΑ ecosystem currently requires developers to read Ancient Greek source files directly to understand an API's structures, traits, and functions. A more accessible, automated documentation generation system is necessary to produce human-readable references without inspecting raw code.

## Decision

We have added the "Scholar" (`ὁ Σχολαστικός`) tool in `src/tools/scholar.rs` to the "Developer Experience (Nova)" toolset. The Scholar tool distills the defined types (Structs), traits (Interfaces), and functions (Verbs) from the semantic analysis output and automatically generates structured, GitHub-flavored Markdown API documentation (`doc.md`).

## Consequences

*   **Positive:** Developers get clear, automated Markdown documentation for their Glossa APIs.
*   **Positive:** Lowers the barrier to entry by removing the need to read Greek source for API usage.
*   **Negative:** Adds a new tool that must be maintained as part of the `glossa` toolset.
