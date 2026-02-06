# 6. Integrate Oracle and Remove Experimental Module

Date: 2024-10-24

## Status

Accepted

## Context

The `src/experimental` directory served as a staging area for the `Oracle` tool, a utility designed to visualize and explain the semantic analysis of Greek sentences.

However, the project adheres to a "Razor" philosophy, which mandates that the codebase should strictly contain production-relevant code. Maintaining a permanent `experimental` folder creates a temptation to merge unfinished or speculative code into the main branch, accumulating technical debt and confusing the module structure.

Despite its location, the `Oracle` tool has proven to be a valuable asset for debugging the `Assembler`. It provides human-readable feedback on how the compiler interprets free word order, case roles, and morphology, which is essential for both developers and users learning the language.

## Decision

We will integrate the `Oracle` tool into the core semantic analysis module and remove the experimental staging area.

1.  **Move** `Oracle` from `src/experimental` to `src/semantic/oracle.rs`.
2.  **Expose** `Oracle` functionality for use in the REPL and CLI for code explanation.
3.  **Delete** the `src/experimental` directory.

## Consequences

*   **Cleanliness:** The project structure is flattened and simplified. All modules in `src/` are now considered part of the production compiler.
*   **Discoverability:** The `Oracle` tool is now a documented component of the Semantic Analysis phase, making its relationship to the `Assembler` explicit.
*   **Workflow:** New experimental features must be developed in dedicated feature branches rather than being committed to a "sandbox" folder in the main codebase.
