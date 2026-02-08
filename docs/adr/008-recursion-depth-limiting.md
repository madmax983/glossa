# 8. Enforce Recursion Limits and Remove Token Caps

Date: 2024-10-25

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler uses a recursive descent parser (via `pest`). While efficient and easy to implement, recursive descent parsers are vulnerable to stack overflows when processing deeply nested structures (e.g., `((((...))))` or deeply nested blocks). Since Rust's default stack size is finite, a sufficiently malicious or accidental input could crash the compiler.

Previously, there was an implicit reliance on limiting the number of tokens fed into the `Assembler` as a proxy for complexity control. However, this approach was flawed because:
1.  **Sentence Length != Nesting Depth:** A very long sentence can be perfectly flat (e.g., a long array literal `[1, 2, ..., 1000]`), while a very short sentence can be deeply nested `((((x))))`.
2.  **Valid Use Cases Blocked:** Arbitrary token limits prevented valid use cases involving long lists of adjectives or large data literals.
3.  **Late Detection:** Token limits were checked during semantic analysis, potentially after the stack had already overflowed during parsing.

## Decision

We have decided to decouple structural safety from sentence length by implementing a dedicated recursion depth check.

1.  **Linear Pre-Scan:** Before parsing begins, the source code undergoes a linear scan (`check_recursion_depth` in `src/parser/builder.rs`) to track the nesting level of parentheses `()`, braces `{}`, and brackets `[]`.
2.  **Strict Limit:** A hard limit of **500** nested levels is enforced. If exceeded, the compiler returns a `RecursionLimitExceeded` error immediately.
3.  **Remove Token Caps:** The `Assembler` (`src/semantic/assembler.rs`) no longer enforces a `MAX_TOKENS` limit. It will process as many tokens as the parser feeds it, allowing for arbitrarily long flat sentences.

## Consequences

*   **Reliability:** The compiler is protected against stack overflow crashes caused by deep nesting.
*   **Performance:** Parsing now incurs an additional O(N) pass over the source code. Given the speed of linear scanning compared to parsing, this overhead is considered negligible for the safety guarantees it provides.
*   **Flexibility:** Users can now write large array literals or complex sentences with many modifiers without hitting artificial "sentence too long" errors.
*   **Error Reporting:** Users receive a clear, early error message about recursion depth rather than a crash or a confusing "too many tokens" error.
