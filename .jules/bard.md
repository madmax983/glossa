# Bard's Journal

## 2024-05-24 - The Hex Encoding Reality
**Confusion:** The documentation for `sanitize_name` in `src/codegen.rs` claimed that single Greek letters like `α` map directly to ASCII `a`. However, the implementation hex-encodes *all* Greek characters (e.g., `α` -> `_u3b1_`) to prevent collisions with existing ASCII identifiers.
**Clarification:** Updated the documentation to reflect the true behavior: all non-ASCII characters are hex-encoded. This ensures that `x` (ASCII) and `ξ` (Greek Xi) never collide in the generated Rust code.

## 2024-05-24 - The Koronis subtlety
**Confusion:** The Greek Koronis (`᾽`, U+1FBD) looks like a breathing mark but behaves like a letter in some contexts. `normalize_greek` treats it as a modifier letter and preserves it, resulting in identifiers like `_u1fbd_` in generated code.
**Clarification:** Documented this behavior in `src/text.rs` and added a doctest to explicitly demonstrate it. Users should be aware that `᾽` is significant in identifiers.
