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
## 2026-03-18 - [The Sentential Assembled Statement]
**Confusion:** The semantic assembler structures, particularly `AssembledStatement` and `Constituent`, lacked any explicit explanation of why they exist as intermediate layers or how they relate to Ancient Greek's word order independence.
**Clarification:** I added robust module-level docs to these structs. The `Constituent` docs now clarify that it bridges raw morphology with syntax verification, while `AssembledStatement` docs explain how it acts as "buckets" for different case roles, enabling word-order flexibility before semantic meaning (e.g. print vs loop vs function call) is inferred.
## 2024-04-03 - Fixing Broken Intra-Doc Links
**Confusion:** The documentation contained broken links to internal private modules (`parser::grammar` and `cache`), causing warnings during `cargo doc`.
**Clarification:** Updated the intra-doc links to point to the exported, public equivalents (`parser` and `Cache`) so that the documentation correctly resolves and is warning-free.
## 2024-04-03 - Fixing Broken Intra-Doc Links
**Confusion:** The documentation contained broken links to internal private modules (`parser::grammar` and `cache`), causing warnings during `cargo doc`.
**Clarification:** Updated the intra-doc links to point to the exported, public equivalents (`parser` and `Cache`) so that the documentation correctly resolves and is warning-free.
## 2024-05-27 - Documenting internal pub fns
**Confusion:** Writing doc-tests for `pub fn` functions inside `pub(crate)` modules (like `analyze_verb_all_into`) fails compilation because doc-tests run as an external crate and cannot access `pub(crate)` items.
**Clarification:** Use ````text` blocks instead of ````rust` for doc-tests on functions that cannot be tested externally due to module visibility, or structure the code so public APIs are testable.

## 2026-03-19 - Broken Intra-Doc Links Resolved
**Confusion:** The `cargo doc` command was throwing warnings about unresolved intra-doc links to `ScopeGuard` in `src/semantic/resolver.rs` and linking to the private struct `GlossaReport` in `src/tools/runner.rs`.
**Clarification:** I replaced the intra-doc link `[`ScopeGuard`]` with `` `ScopeGuard` `` because the type itself was not explicitly defined (maybe handled as a closure via `with_scope`). For `GlossaReport`, since it is not public in `tools::mod.rs` by default without features/exports, I replaced the intra-doc link with `` `GlossaReport` `` to avoid linking issues to private types in public function docs.
## 2026-03-19 - [Empty Code Blocks in Assembly Structs]
**Confusion:** The structs `AssembledStatement` and `Constituent` in `src/semantic/assembly.rs` had placeholder documentation containing empty rust code blocks, leading to `cargo doc` warnings ("Rust code block is empty").
**Clarification:** I populated these documentation blocks with executable, concrete `doctests` demonstrating how to construct these types using realistic Ancient Greek morphology examples and explicit `crate::` path imports, providing tangible usage context while eliminating compiler warnings.

## 2026-03-19 - The Missing Link in Tools
**Confusion:** The `src/tools/` sub-modules like `runner.rs`, `papyrus.rs`, and `auditor.rs` lacked `//!` module-level documentation. This created "The Black Box" where complex modules were missing a high-level conceptual overview.
**Clarification:** Added comprehensive `//!` documentation blocks to `src/tools/runner.rs`, `src/tools/papyrus.rs`, and `src/tools/auditor.rs`, explaining *what* they do and *how* they work. Also added executable doc tests to `analyze_source` and `load_source` to demonstrate usage.
## 2026-03-20 - [The Assembler Documentation Duplication]
**Confusion:** The documentation for the `Assembler` struct in `src/semantic/assembly.rs` was duplicated three times, causing a wall of text that was redundant and confusing.
**Clarification:** I deleted the duplicated documentation blocks, leaving only one clean, descriptive rustdoc comment with its code example. This makes the generated HTML documentation much easier to read and maintain.
## 2026-03-21 - [Missing Documentation Warnings]
**Confusion:** Building documentation with `RUSTDOCFLAGS="-W missing_docs" cargo doc` threw warnings for `src/semantic/assembly/mod.rs` regarding the `model` module, and the compiler output warnings for missing intra-doc links.
**Clarification:** Added module-level documentation `//!` to `src/semantic/assembly/model.rs` and moved the struct documentation for `Assembler` immediately above its definition. Resolved intra-doc links by ensuring they reference paths accessible in scope (e.g. `[`crate::semantic::assembly::Assembler`]`).

## 2026-03-21 - missing_docs in parser
**Confusion:** The documentation for `parse_number_literal` was missing an `# Examples` section but had no warnings.
**Clarification:** I added the required `# Examples` section.
## 2026-03-22 - Missing Documentation in `gnomon.rs`
**Confusion:** The `src/tools/gnomon.rs` file had a module-level documentation block (`//!`) but the `GnomonVisitor` struct, its methods, and the `run_gnomon` function were completely undocumented, resulting in missing information on what the tool actually did or how it achieved it.
**Clarification:** Added exhaustive documentation explaining that the visitor casts a "shadow" over the AST to check for max depth of loops (complexity). Wrote an executable doctest for `run_gnomon` to demonstrate its usage.
## 2026-05-03 - The Scholar Tool's Missing Link
**Confusion:** The `src/tools/scholar.rs` module lacked module-level documentation and an executable doc-test for its public `run_scholar` function. It was not telling a story of *why* it existed, only what it was called, making it a "Black Box".
**Clarification:** Added a comprehensive module-level `//!` documentation block that explicitly outlines the "Missing Link" and explains the philosophy behind automatically generating Markdown API docs from AST definitions. Added an executable `## Examples` block to `run_scholar`.
## 2026-06-13 - The Misplaced Import Doc Stealer
**Confusion:** The documentation for `to_rust_type` in `src/codegen.rs` was throwing a `missing_docs` warning, even though it had a massive rustdoc block right above it.
**Clarification:** The `use std::fmt::Write;` statement was placed *between* the doc block and the function. In Rust, doc comments attach to the immediately following item. By separating the doc block and the function with a `use` statement, the documentation was attached to the import instead of the public function. Moved the import statement to resolve the issue.
