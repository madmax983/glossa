# 16. Centralize MAX_EXPRESSION_DEPTH limit

Date: 2026-03-06
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler is susceptible to denial-of-service (DoS) attacks via stack overflows when parsing deeply nested expressions. While a recursion depth limit for the parser (`MAX_PARSE_DEPTH`) was already implemented and centralized, the limit during the semantic analysis phase (`MAX_EXPRESSION_DEPTH`) remained scattered and hardcoded within modules like `src/semantic/expressions.rs`. This lack of centralization made the system's structural limits difficult to audit and maintain, violating the Atlas persona's architectural directives for high cohesion and single sources of truth.

## Decision

We have decided to centralize the semantic depth limit (`MAX_EXPRESSION_DEPTH`, currently set to 200) into the `src/limits.rs` module, alongside the existing parser limits. Furthermore, the AST depth validation logic has been extracted into a dedicated `src/semantic/validation.rs` module to separate tree limit checks from semantic orchestration.

## Consequences

*   **Maintainability**: All compiler-wide architectural limits (recursion depths, etc.) are now located in a single, auditable file (`src/limits.rs`).
*   **Security**: Centralized limits ensure that protective bounds against stack overflows and DoS attacks are consistently applied and easier to tune.
*   **Structural Clarity**: Extracting the validation logic into `src/semantic/validation.rs` improves the cohesion of the `semantic` module, removing limit-checking boilerplate from the core expression analysis paths.
