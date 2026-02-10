# 9. Promote Experimental Bard to Core Highlight Module

Date: 2024-10-25

## Status

Proposed

## Context

The `src/experimental` directory persists despite ADR 006. The `bard` module provides valuable syntax highlighting functionality but is hidden in `experimental`. This violates the Razor philosophy of "no experimental code in main".

ADR 006 intended to integrate `Oracle` and remove `experimental`, but the implementation was incomplete or `Oracle` was removed entirely while `bard` remained.

## Decision

We will promote the `bard` module to a top-level `highlight` module and remove the `experimental` directory entirely.

1.  **Move** `src/experimental/bard.rs` to `src/highlight.rs`.
2.  **Delete** the `src/experimental` directory.
3.  **Update** `src/lib.rs` and `src/main.rs` to reference the new module.

## Consequences

*   **Cleanliness:** The project structure is flattened and simplified, adhering to the Razor philosophy.
*   **Discoverability:** The syntax highlighting functionality is now a documented, first-class component of the compiler.
*   **Consistency:** The codebase aligns with the architectural vision of having no permanent experimental modules.
