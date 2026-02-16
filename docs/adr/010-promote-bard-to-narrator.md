# 10. Promote Bard to Narrator Tool

Date: 2024-10-24

## Status

Accepted

## Context

The `bard` module, currently located in `src/experimental/bard.rs`, provides functionality to translate the semantic analysis of a ΓΛΩΣΣΑ program into a human-readable English narrative. This feature, exposed via the `tell_tale` function, is invaluable for debugging the compiler's understanding of code and for educational purposes, helping users understand how their Greek code is interpreted.

However, the existence of the `src/experimental` directory as a permanent staging area contradicts the project's "Razor" philosophy, which mandates that the codebase should not accumulate tentative or "sandbox" code. Features should either be fully integrated into the production compiler or removed. The ambiguous status of `bard` leads to questions about its stability and maintenance.

## Decision

We will promote the `bard` functionality to a first-class tool within the compiler's toolset.

1.  **Move** the contents of `src/experimental/bard.rs` to a new module `src/tools/narrator.rs`.
2.  **Rename** the module from `bard` to `narrator` to better reflect its function (generating a narrative).
3.  **Remove** the `src/experimental` directory entirely.
4.  **Update** the `Runner` and CLI to use the new `narrator` module.

## Consequences

*   **Structural Clarity:** The removal of `src/experimental` enforces the rule that all code in `src/` is production-quality.
*   **improved Maintenance:** As a core tool, `narrator` will be subject to the same testing and documentation standards as other tools like `highlight` or `check`.
*   **Terminology:** The term "Narrator" aligns well with the "Hero's Journey" theme of the documentation, serving as the component that tells the story of the code.
