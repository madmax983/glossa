# 9. Promote Bard and Cleanup Semantic Module

Date: 2024-10-24

## Status

Accepted

## Context

The codebase has evolved since the initial experimental phase. Several structural issues have emerged:

1.  **Experimental Directory**: The `src/experimental` directory contained the `bard` syntax highlighter. This module has proven useful and stable, but its location suggests it is not part of the core product. The "Razor" philosophy requires removing unnecessary nesting and experimental staging areas once features are mature.
2.  **Semantic Module fragmentation**: The `src/semantic` module separated `AssembledStatement` (the data structure) into `assembled.rs` and `Assembler` (the logic) into `assembler.rs`. This separation created circular dependencies in thought processes and cluttered imports, as the two are inextricably linked.
3.  **Obsolete Components**: The `Oracle` component (intended for explaining code) was superseded or removed from the code but remained in the documentation.
4.  **Reporting**: A new `report` module was introduced to handle CLI output statistics but was not formally recognized in the architecture.

## Decision

We will perform a structural cleanup to align the codebase with the "Razor" philosophy (Essentialism):

1.  **Promote Bard**: Move `src/experimental/bard.rs` to `src/highlight.rs`. Expose it as `glossa::highlight`.
2.  **Remove Experimental**: Delete the `src/experimental` directory.
3.  **Merge Semantic Assembly**: Merge `src/semantic/assembled.rs` into `src/semantic/assembler.rs`. The `AssembledStatement` struct is now defined alongside the `Assembler` that produces it.
4.  **Remove Oracle**: Officially deprecate and remove the `Oracle` component from the architecture.
5.  **Adopt Report**: Recognize `src/report.rs` as a core component for CLI feedback.

## Consequences

*   **API Visibility**: `glossa::highlight` is now a first-class citizen of the library, making it easier for external tools (like IDE plugins) to use the semantic highlighter.
*   **Simpler Hierarchy**: The source tree is flatter. `src/semantic` has fewer files, and `src/experimental` is gone.
*   **Cohesion**: The `Assembler` logic and its data structures are co-located, improving code navigability.
*   **Accuracy**: The architecture documentation will now accurately reflect the codebase state (no missing `Oracle`, present `Report`).
