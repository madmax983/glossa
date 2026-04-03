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

## 2025-03-05 - The `cargo test --doc` Dependency Trap
**Confusion:** Running `rustdoc --test src/codegen.rs` manually failed with unresolved import errors for internal modules like `morphology`, `semantic`, and external crates like `proc_macro2` and `smol_str`. It seemed as if the doc tests were missing imports.
**Clarification:** Doc tests inside a crate are compiled as external black-box tests. When using `cargo test --doc`, Cargo correctly sets up the environment with `extern crate glossa` and all external dependencies available, meaning you just use `glossa::...` and external crates resolve automatically. Manual `rustdoc` runs lack this crate resolution context. Doc tests should only be verified with `cargo test --doc`.

## 2025-03-11 - The Intra-Doc Link Warning Dilemma
**Confusion:** The compiler output a warning `public documentation for Scope links to private item ScopeLevel`. Attempting to fix this by making `ScopeLevel` public violates the principle of keeping internal implementations hidden. Attempting to fix it by removing the intra-doc link (`[ScopeLevel]`) violates the Bard directive to use clickable links.
**Clarification:** You should not expose internal private structs as public API solely to resolve `cargo doc` intra-doc link warnings. Making internal implementation details public is considered an anti-pattern. If a link points to a private item, the warning is acceptable if the alternative compromises the public API surface or explicitly contradicts persona guidelines. In this case, `ScopeLevel` remains private and the warning is tolerated.

## 2025-03-11 - Cohesive Doc Block Structure
**Confusion:** Adding new sections (like "Why it exists" and "Examples") to existing doc blocks can lead to disjointed reading experiences (e.g., an example, followed by a summary, followed by another example) if the existing block structure is not fully considered.
**Clarification:** When updating existing rustdoc comments with new sections, reorganize the entire doc block so the top-level summary is at the very beginning, followed by explanatory sections, and ending with the code examples. Do not simply append to the end of an existing block if it breaks the logical flow.

## 2025-03-11 - Constructing Errors Consistently
**Confusion:** The constructors for `GlossaError` (`parse`, `semantic`, `codegen`, etc.) in `src/errors/mod.rs` were missing rustdoc comments, leading to confusion about when to use which constructor and how they map to the underlying enum variants.
**Clarification:** I added cohesive rustdoc to all public error constructors. Each block now explains *what* the constructor makes, *why* you would use it (e.g., syntactical invalidity vs logical invalidity vs internal compiler bug), and provides an executable example demonstrating its usage.
## 2026-03-18 - [The Semantic Model]
**Confusion:** Lack of documentation for the semantic models in  which serves as the core data container.
**Clarification:** Added comprehensive module-level documentation explaining the Atlas pattern, the separation of logic, types, and state, and included a code example illustrating an analyzed binding statement.
## 2026-03-18 - [The Semantic Model]
**Confusion:** Lack of documentation for the semantic models in `src/semantic/model.rs` which serves as the core data container.
**Clarification:** Added comprehensive module-level documentation explaining the Atlas pattern, the separation of logic, types, and state, and included a code example illustrating an analyzed binding statement.
## 2024-04-03 - Fixing Broken Intra-Doc Links
**Confusion:** The documentation contained broken links to internal private modules (`parser::grammar` and `cache`), causing warnings during `cargo doc`.
**Clarification:** Updated the intra-doc links to point to the exported, public equivalents (`parser` and `Cache`) so that the documentation correctly resolves and is warning-free.
