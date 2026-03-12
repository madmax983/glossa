# 16. Centralize Compiler Limits

Date: 2026-03-03

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler incorporates various structural and semantic limits to prevent Denial of Service (DoS) vulnerabilities, such as stack overflows from deeply nested expressions or excessive memory consumption from large collections. Previously, these constants (like `MAX_RECURSION_DEPTH`) and validation logic were scattered across different modules (`src/parser/recursion.rs`, `src/semantic/expressions.rs`). This scattering created "magic numbers," disconnected logic, and made it difficult to audit or tune the compiler's safety constraints as a cohesive unit.

Furthermore, relying on localized checks during parsing or assembly was insufficient to guard against complex semantic structures that might pass parsing but cause overflows during AST evaluation.

## Decision

We have decided to centralize all compiler limits into a single source of truth and extract AST depth validation into its own dedicated semantic module.

1.  **Centralized Limits:** Created a new module `src/limits.rs` that explicitly defines all compiler-wide limits (e.g., `MAX_PARSE_DEPTH` (now 250), `MAX_AST_DEPTH`, `MAX_EXPRESSION_DEPTH` (200), and various semantic assembly constraints).
2.  **Dedicated Validation Component:** Extracted AST depth validation from scattered locations into a dedicated `src/semantic/validation.rs` module.
3.  **Encapsulation:** The parser's internal submodule structure was refactored to hide implementation details where possible, with limits and validation now operating through explicitly imported constants.

## Consequences

*   **Auditability:** All architectural limits are now defined in a single file (`src/limits.rs`), making them easy to review, modify, and document.
*   **Security:** By extracting AST depth validation into `validation.rs`, we ensure a consistent and isolated pass over the generated `AnalyzedProgram` to enforce constraints before code generation or interpretation.
*   **Decoupling:** Modules across the parser and semantic analyzer no longer hardcode magic numbers or duplicate validation logic, adhering to the Single Responsibility Principle.
*   **Documentation:** Architecture diagrams have been updated to reflect the new `MAX_PARSE_DEPTH` of 250 and to include the `Validation` component in the Semantic Analysis C4 model.
