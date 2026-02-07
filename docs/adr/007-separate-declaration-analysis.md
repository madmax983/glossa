# 7. Separate Declaration Analysis from Statement Assembly

Date: 2024-10-25

## Status

Accepted

## Context

The Semantic Analyzer originally processed all input through the `Assembler` pipeline, which is designed for the language's signature free word order. While this works well for imperative statements (like function calls, assignments, and prints), declarative structures (such as Type Definitions, Trait Definitions, and Test Declarations) often follow a more rigid structure or require specialized handling that doesn't benefit from the Assembler's flexibility.

Attempting to force declarations through the Assembler led to:
1.  **Complexity:** The Assembler had to handle disparate concerns (e.g., distinguishing between a function call and a type definition).
2.  **Scattered Logic:** Type resolution and validation logic was dispersed across multiple modules.
3.  **Performance:** Running fixed-structure declarations through the expensive word-order permutation logic of the Assembler was inefficient.

## Decision

We have decided to architecturally separate the analysis of Declarations from the analysis of Imperative Statements.

1.  **Introduce `src/semantic/declarations.rs`:** A new module dedicated to analyzing Type Definitions, Trait Definitions, Trait Implementations, and Test Declarations.
2.  **Dispatch in `analyze_program`:** The main entry point `analyze_program` now acts as a high-level dispatcher. It inspects the AST statement kind and routes:
    -   **Declarations** -> `declarations.rs` (Direct analysis).
    -   **Imperative Statements** -> `Assembler` + `Converter` (Word-order independent assembly).
3.  **Centralize Type Resolution:** The logic for resolving type names (including Greek genitive mapping like `ἀριθμοῦ` → `Number`) is centralized in `declarations::resolve_type_name`.

## Consequences

*   **Separation of Concerns:** The `Assembler` is now focused solely on assembling imperative sentences, making it simpler and more robust.
*   **Maintainability:** Declaration logic is isolated in a single module, making it easier to add new declaration types or modify existing ones.
*   **Clarity:** The distinction between "defining structure" (Declarations) and "executing logic" (Statements) is explicit in the compiler pipeline.
*   **Extensibility:** Future declarative features can be added to `declarations.rs` without touching the complex Assembler logic.
