# 11. Consolidate Semantic Analysis Modules

Date: 2024-10-27

## Status

Proposed

## Context

The initial design (ADR 007) separated Declaration Analysis (`declarations.rs`) from Imperative Statement Assembly (`assembler.rs`). This was intended to handle the rigid structure of declarations versus the free word order of imperative statements.

However, as the language evolved, several issues emerged:
1.  **Shared Logic:** Both declarations (like function definitions) and imperative statements (like `if`, `while`) require expression analysis and scope management.
2.  **Blurred Lines:** Function definitions are technically statements that can appear inside blocks, blurring the distinction between "declarations" and "statements".
3.  **Duplication:** Logic for parsing expressions and managing scopes was duplicated or awkwardly shared.
4.  **Complexity:** Having a separate `declarations.rs` module for some constructs and `assembler.rs` for others led to a fragmented analysis pipeline.

## Decision

We have decided to consolidate the semantic analysis structure:

1.  **Merge Declarations into Statements:** The `src/semantic/declarations.rs` module is removed. Its logic is merged into `src/semantic/statements.rs`, which now handles all high-level statement analysis, including Control Flow (If, While) and Declarations (Type, Trait, Function, Test).
2.  **Extract Expression Analysis:** Recursive descent logic for nested expressions (function calls, arithmetic) is formalized in `src/semantic/expressions.rs`. This module is used by both `statements.rs` (for conditions, initializers) and `assembler.rs` (for feeding arguments).
3.  **Formalize Resolver:** Scope management and name resolution logic is centralized in `src/semantic/resolver.rs`. This module manages the symbol table, scope stack, and type/trait lookups.

## Consequences

*   **Simplified Structure:** The semantic analysis module is more cohesive, with `statements.rs` acting as the primary orchestrator for non-assembler constructs.
*   **Centralized Logic:** Expression parsing and scope management are centralized, reducing duplication and bugs.
*   **Explicit Dependencies:** The relationships between modules are clearer: `Statements` depends on `Expressions` and `Resolver`; `Assembler` depends on `Expressions`.
*   **Documentation Update:** Architecture diagrams must be updated to reflect the removal of `declarations` and the prominence of `statements`, `expressions`, and `resolver`.
