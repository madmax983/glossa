# 9. Flatten Module Hierarchy

Date: 2024-05-22

## Status

Accepted

## Context

The codebase was using deeply nested directory structures for modules (`src/ast/mod.rs`, `src/grammar/mod.rs`, `src/errors/mod.rs`) even when the module content could fit in a single file or a flatter structure.
This violates the "Razor" principle of essentialism and adds friction to navigation and refactoring.
Having to open `src/ast/mod.rs` to see the module definition is less direct than `src/ast.rs`.

## Decision

We decided to flatten the following modules:
*   `src/ast/` -> `src/ast.rs`
*   `src/grammar/` -> `src/grammar.rs` (and move `glossa.pest` to `src/grammar.pest`)
*   `src/errors/` -> `src/errors.rs`

## Consequences

*   **Positive**:
    *   Simpler file structure.
    *   Easier to see at a glance what modules exist.
    *   Reduced boilerplate (no `mod.rs` files for simple modules).
*   **Negative**:
    *   If a module grows significantly, it might need to be split again (but we can use `mod submodule;` within the file instead of folders).
