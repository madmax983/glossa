# 11. Refactor Semantic Analysis Pipeline

Date: 2024-05-20

## Status

Accepted

## Context

The initial implementation of the semantic analysis phase was monolithic, relying heavily on a single `declarations.rs` module to handle both top-level declarations (types, functions) and statement analysis. The core "Assembler" logic—designed to handle the language's free word order—was tightly coupled with statement conversion.

As the language evolved to support more complex features, several issues emerged:
1.  **Recursion Difficulty**: Implementing recursive expressions (e.g., `f(g(x))` or `1 + 2 * 3`) was difficult because the flat assembler structure wasn't designed for nested trees.
2.  **Scope Management**: Variable scoping, shadowing, and function resolution were scattered across multiple files, leading to inconsistencies.
3.  **Control Flow Complexity**: Adding `if`, `while`, and `match` statements required a more structured approach than the simple "slot-filling" mechanism of the assembler.

## Decision

We will refactor the semantic analysis pipeline into specialized modules with distinct responsibilities:

1.  **Statements (`src/semantic/statements.rs`)**:
    *   Handles high-level statement analysis.
    *   Responsible for parsing control flow constructs (`if`, `while`, `for`, `match`).
    *   Responsible for parsing declarations (`Type`, `Trait`, `Function`, `Test`).
    *   Acts as the controller for analyzing blocks of code.

2.  **Expressions (`src/semantic/expressions.rs`)**:
    *   Handles recursive expression analysis.
    *   Manages operator precedence and nested function calls.
    *   Feeds simple terms back into the assembler when needed for slot filling.

3.  **Resolver (`src/semantic/resolver.rs`)**:
    *   Centralizes scope management.
    *   Handles name resolution for variables, functions, types, and traits.
    *   Manages variable shadowing and scope nesting (e.g., function bodies, loops).

4.  **Assembler (`src/semantic/assembler.rs`)**:
    *   Remains focused on the core "slot-based" assembly of simple Subject-Verb-Object sentences.
    *   Agnostic to high-level control flow.

5.  **Conversion (`src/semantic/conversion.rs`)**:
    *   Handles the conversion of filled assembler slots into typed `AnalyzedStatement`s.
    *   Bridges the gap between the raw grammatical structure and the semantic model.

## Consequences

**Positive:**
*   **Separation of Concerns**: Each module has a clear, single responsibility.
*   **Testability**: Expression recursion and control flow logic can be tested independently of the slot-based assembler.
*   **Scalability**: Adding new control flow constructs or expression types is localized to specific modules.
*   **Scope Clarity**: All name resolution logic is centralized in `resolver.rs`, reducing bugs related to variable visibility.

**Negative:**
*   **Complexity**: The interaction between `statements` (for structure) and `assembler` (for content) is more complex than a single pass.
*   **Interdependencies**: Modules now have circular dependencies (e.g., `statements` uses `expressions`, which might use `assembler`, which might need context from `resolver`).

This architecture aligns with the "Slot-Based Semantic Assembly" (ADR 004) while extending it to support a full-featured programming language.
