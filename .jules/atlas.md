## [Breaking The Knot: Semantic-IR Cycle]
**Tangle:** The `semantic` module imports `ir` to use `IteratorOp` and `lower_expr` for detecting iterator patterns. The `ir` module imports `semantic` to consume `AnalyzedProgram` for lowering. This creates a circular dependency `semantic <-> ir`.
**Blueprint:**
1. Define `AnalyzedIteratorOp` in `semantic` to mirror `IteratorOp` but with `AnalyzedExpr`.
2. Update `AnalyzedExpr` to use `AnalyzedIteratorOp`.
3. Stop calling `ir::lower_expr` inside `semantic`.
4. Update `ir` to map `AnalyzedIteratorOp` to `ir::IteratorOp` during lowering.
**Stability:** Decouples semantic analysis from IR generation details. `semantic` becomes strictly upstream of `ir`.

## [Splitting The Blob: Semantic Module]
**Tangle:** `src/semantic/mod.rs` had grown to over 4600 lines, mixing high-level orchestration, control flow parsing, pattern matching, and expression analysis. This violated the Single Responsibility Principle and made navigation difficult.
**Blueprint:** Split `src/semantic/mod.rs` into `control_flow.rs`, `declarations.rs`, `expressions.rs`, `patterns.rs`, and `conversion.rs`. `mod.rs` now acts as an orchestrator and facade.

## [Breaking The Knot: Semantic Types and Models]
**Tangle:** `src/semantic/types.rs` depended on `AnalyzedStatement` (from `mod.rs`) for method bodies, while `mod.rs` depended on `GlossaType` (from `types.rs`). This created a circular dependency where the Type System depended on the AST it was supposed to type.
**Blueprint:**
1. Created `src/semantic/model.rs` to hold all AST nodes (`AnalyzedStatement`, `AnalyzedExpr`) and Semantic Models (`TraitDef`, `TraitImpl`).
2. Moved `TraitDef` and friends from `types.rs` to `model.rs`.
3. Moved AST nodes from `mod.rs` to `model.rs`.
4. Refactored `mod.rs` to re-export `model` contents.
5. Removed legacy `SemanticAnalyzer` code that was duplicating logic.
**Stability:** Strictly separates Data (Model), Types (Type System), and Logic (Analysis). `model.rs` depends on `types.rs`, but `types.rs` is now leaf-level with no dependencies on the AST.

## [Breaking The Leak: Semantic Assembly Error]
**Tangle:** `src/semantic/assembler.rs` defined `AssemblyError`, but `src/semantic/mod.rs` was stringifying it into `GlossaError::SemanticError` to avoid circular dependencies (since `errors` depends on `semantic` if `GlossaError` wraps `AssemblyError`). This caused loss of structured error information.
**Blueprint:**
1. Created `src/errors/assembly.rs` and moved `AssemblyError` there.
2. Updated `GlossaError` to include `AssemblyError` as a transparent variant.
3. Updated `src/semantic/assembler.rs` to use the shared error type.
**Stability:** Centralizes error definition, preserves error structure for better diagnostics, and maintains acyclic graph (`errors` -> `morphology`, `semantic` -> `errors`).

## [Breaking The Knot: Grammar Module Coupling]
**Tangle:** The `grammar` module (parser) was exposing `normalize_greek`, a text utility. Every other module (`ast`, `morphology`, `semantic`, `codegen`) depended on `grammar` solely for this function. This coupled the entire compiler to the parser implementation details, even for modules that should be lower-level (like `ast` and `morphology`).
**Blueprint:**
1. Extracted `normalize_greek` to a new `src/text.rs` module.
2. Updated all imports to point to `crate::text::normalize_greek`.
3. Removed `normalize` submodule from `grammar`.
**Stability:** Decouples core logic from parsing implementation. `text` becomes a leaf utility module, allowing `ast` and `morphology` to be independent of `grammar`.

## [Breaking The Leak: Semantic Expressions]
**Tangle:** `src/semantic/expressions.rs` was exposing internal logic (`feed_expr_to_assembler_with_context`) via `#[doc(hidden)]` solely for the `Oracle` tool in `src/experimental/oracle.rs`. This violated encapsulation and forced `Oracle` to rely on internal implementation details of the assembler.
**Blueprint:**
1. Exposed `assemble_statement` in `src/semantic/mod.rs` as a high-level facade API.
2. Refactored `Oracle` to use `assemble_statement` instead of manually driving the assembler loop.
3. Changed `src/semantic/expressions.rs` visibility to `pub(crate)` to enforce internal-only use.
**Stability:** Enforces clear module boundaries. `semantic` now provides a stable public API for assembly, and internal expression logic is hidden from external consumers.

## [Breaking The Knot: Parser-Errors Cycle]
**Tangle:** The `errors` module imported `parser::ParseError` to implement the `From` trait for `GlossaError`, while the `parser` module imported `GlossaError` as its return type. This created a circular dependency `errors <-> parser`.
**Blueprint:**
1. Moved the `impl From<ParseError> for GlossaError` block from `src/errors/mod.rs` to `src/parser/mod.rs`.
2. Removed the dependency on `crate::parser` from `src/errors/mod.rs`.
**Stability:** `errors` is now a lower-level module that does not depend on `parser`. The dependency graph is strictly `parser -> errors`.

## [Breaking The Leak: Experimental Module]
**Tangle:** The `src/experimental` module contained `bard.rs` (Syntax Highlighter), a stable feature used in the CLI (`main.rs`). This violated module boundaries by exposing "experimental" code in production and acting as a catch-all dumping ground.
**Blueprint:**
1. Moved `src/experimental/bard.rs` to `src/highlight.rs`.
2. Updated `src/lib.rs` and `src/main.rs` to use the new module.
3. Removed `src/experimental`.
**Stability:** Promotes the Syntax Highlighter to a first-class citizen, flattens the module hierarchy, and removes the unstable `experimental` module.
