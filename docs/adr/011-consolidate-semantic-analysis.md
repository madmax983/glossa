# 11. Consolidate Semantic Analysis Modules

Date: 2024-10-27

## Status

Accepted

## Context

The initial design (ADR 007) separated Declaration Analysis (`declarations.rs`) from Imperative Statement Assembly (`assembler.rs`). This was intended to handle the rigid structure of declarations versus the free word order of imperative statements.

However, as the language evolved, several issues emerged:
1.  **Shared Logic:** Both declarations (like function definitions) and imperative statements (like `if`, `while`) require expression analysis and scope management.
2.  **Blurred Lines:** Function definitions are technically statements that can appear inside blocks, blurring the distinction between "declarations" and "statements".
3.  **Duplication:** Logic for parsing expressions and managing scopes was duplicated or awkwardly shared.
4.  **Complexity:** Having a separate `declarations.rs` module for some constructs and `assembler.rs` for others led to a fragmented analysis pipeline.

## Decision

We have decided to consolidate the semantic analysis structure, with some modifications to the initial proposal:

1.  **Orchestrator Pattern:** Instead of a monolithic `statements.rs`, the `src/semantic/mod.rs` module acts as the orchestrator (via `analyze_statement`), delegating to specialized modules.
2.  **Specialized Modules:**
    -   `declarations.rs` is **retained** to handle Type, Trait, Function, and Test definitions.
    -   `control_flow.rs` handles If, While, Match logic.
    -   `conversion.rs` handles the conversion of assembled statements.
3.  **Extract Expression Analysis:** Recursive descent logic for nested expressions (function calls, arithmetic) is formalized in `src/semantic/expressions.rs`. This module is used by `declarations`, `control_flow`, and `assembly`.
4.  **Formalize Resolver:** Scope management and name resolution logic is centralized in `src/semantic/resolver.rs`. This module manages the symbol table, scope stack, and type/trait lookups.

## Consequences

*   **Modular Orchestration:** The `mod.rs` orchestrator keeps the high-level logic clean while delegating complexity to specialized submodules.
*   **Centralized Logic:** Expression parsing and scope management are centralized, reducing duplication and bugs.
*   **Explicit Dependencies:** The relationships between modules are clearer: `mod.rs` depends on `declarations`, `control_flow`, and `conversion`; these depend on `expressions` and `resolver`.
*   **Documentation Update:** Architecture diagrams have been updated to reflect the retention of `declarations`, the addition of `control_flow`, and the central role of `mod.rs`.
*   **Note:** This implementation diverges from the original proposal (which suggested merging everything into `statements.rs`) to maintain better file-level separation of concerns.
