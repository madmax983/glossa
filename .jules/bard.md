# Bard's Journal

## 2024-05-24 - The Hex Encoding Reality
**Confusion:** The documentation for `sanitize_name` in `src/codegen.rs` claimed that single Greek letters like `α` map directly to ASCII `a`. However, the implementation hex-encodes *all* Greek characters (e.g., `α` -> `_u3b1_`) to prevent collisions with existing ASCII identifiers.
**Clarification:** Updated the documentation to reflect the true behavior: all non-ASCII characters are hex-encoded. This ensures that `x` (ASCII) and `ξ` (Greek Xi) never collide in the generated Rust code.

## 2024-05-24 - The Koronis subtlety
**Confusion:** The Greek Koronis (`᾽`, U+1FBD) looks like a breathing mark but behaves like a letter in some contexts. `normalize_greek` treats it as a modifier letter and preserves it, resulting in identifiers like `_u1fbd_` in generated code.
**Clarification:** Documented this behavior in `src/text.rs` and added a doctest to explicitly demonstrate it. Users should be aware that `᾽` is significant in identifiers.

## 2024-05-25 - The Integration Test Boundary
**Confusion:** Developers often try to write tests for `pub(crate)` functions in the `tests/` directory, causing compilation errors like `function is private`.
**Clarification:** `tests/` treats the crate as an external dependency, so it can only access `pub` items. Internal logic tests must live inside the `src/` directory (e.g., in a `mod tests` block). I moved `regression_operator_drop.rs` into `src/semantic/expressions.rs` to fix this.

## 2024-05-26 - The Two Parsers
**Confusion:** Users and contributors often confuse `crate::parser::grammar::parse` (which returns a raw `pest` Parse Tree of generic pairs) with `crate::parser::parse` (which consumes that tree and returns the strongly-typed AST). Calling the wrong one leads to confusing type errors about `Pairs` vs `Program`.
**Clarification:** I added clear documentation and executable examples to `src/parser/grammar.rs` to explicitly state that it returns a Concrete Syntax Tree (CST) and should rarely be used directly by end-users. The module-level docs now explicitly link to `crate::parser::parse` as the preferred entry point for AST generation.
