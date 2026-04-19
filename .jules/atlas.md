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

## [Structuring The Tools: Highlight and Parser Encapsulation]
**Tangle:** `src/highlight.rs` was a top-level file cluttering the root, and `parser::builder` exposed internal implementation details via `pub mod`.
**Blueprint:**
1. Moved `src/highlight.rs` to `src/tools/highlight.rs` and created `src/tools/mod.rs`.
2. Changed `pub mod builder` to `pub(crate) mod builder` in `src/parser/mod.rs`.
3. Updated `src/lib.rs` to export `tools` and re-export `highlight` for compatibility.
**Stability:** Improves module organization by grouping tools and enforcing better encapsulation in the parser module.

## [Refactoring The Monolith: Main Module Sprawl]
**Tangle:** `src/main.rs` was a "God File" containing over 1000 lines of mixed responsibilities: CLI argument parsing, REPL implementation, file compilation/caching logic, and test suites. This violated the Single Responsibility Principle and made the entry point hard to maintain.
**Blueprint:**
1. Created `src/tools/cli.rs` for `clap` argument definitions.
2. Created `src/tools/repl.rs` for the interactive shell logic and state management.
3. Created `src/tools/runner.rs` for the compilation, execution, and caching logic.
4. Refactored `src/main.rs` to be a minimal entry point that delegates to these modules.
**Stability:** Decouples the CLI interface from the core logic. `main.rs` is now a thin wrapper, and each tool component is testable in isolation.

## [Splitting The Blob: Statements Module]
**Tangle:** `src/semantic/statements.rs` was a "Blob" mixing control flow parsing (if, while) with declaration parsing (type, trait, function). This violated Single Responsibility Principle.
**Blueprint:** Split `src/semantic/statements.rs` into `control_flow.rs` and `declarations.rs`. `src/semantic/mod.rs` now orchestrates dispatching to `declarations` for function definitions before handing off to `control_flow`.
**Stability:** Improves cohesion by separating declarations from logic flow. `mod.rs` acts as a clear orchestrator.

## [Splitting The Blob: Assembler]
**Tangle:** `src/semantic/assembler.rs` was a monolithic file (~2000 lines) mixing DTOs (`AssembledStatement`, `Constituent`) with complex assembly logic (`Assembler`) and tests.
**Blueprint:**
1. Created `src/semantic/assembly/` module.
2. Extracted DTOs to `src/semantic/assembly/model.rs`.
3. Moved logic to `src/semantic/assembly/mod.rs`.
4. Updated dependent modules to import from `crate::semantic::assembly`.
**Stability:** Improves separation of concerns (Data vs Logic) and reduces file size.

## [Breaking The Cycle: Semantic Mod vs Submodules]
**Tangle:** The `src/semantic/mod.rs` module was orchestrating analysis but also depending on submodules like `control_flow.rs` and `declarations.rs`. These submodules in turn depended on `analyze_statement` from `mod.rs`, creating a circular dependency that made the module structure fragile and coupled.
**Blueprint:**
1.  Extracted the analysis logic into a new `src/semantic/analyzer.rs` module with a `SemanticAnalyzer` struct.
2.  Defined a `StatementAnalyzer` trait in `src/semantic/traits.rs` to abstract the recursion.
3.  Updated `control_flow.rs` and `declarations.rs` to accept `&mut impl StatementAnalyzer` instead of calling a concrete function.
4.  Re-exported the public API from `mod.rs` to maintain backward compatibility.
**Stability:** Broken the dependency cycle. The dependency graph is now a DAG: `mod` -> `analyzer` -> `control_flow` -> `traits`. Submodules are now leaf-like with respect to the analyzer.

## [Centralizing The Limits: Magic Numbers]
**Tangle:** Hardcoded constants for recursion limits (`MAX_RECURSION_DEPTH`) were scattered across `src/parser/recursion.rs` and `src/semantic/expressions.rs`, leading to disconnected logic and test fragility.
**Blueprint:**
1. Created `src/limits.rs` to centralize all compiler-wide limits.
2. Defined `MAX_PARSE_DEPTH` and `MAX_AST_DEPTH` explicitly.
3. Updated parser, semantic analysis, and tests to import from `crate::limits`.
**Stability:** Enforces a single source of truth for architectural limits, making them easier to audit and tune.

## [Breaking The Leak: Encapsulation via pub(crate)]
**Tangle:** Internal modules were needlessly exposed via `pub mod` across `src/lib.rs`, `src/tools/mod.rs`, `src/parser/mod.rs`, `src/semantic/mod.rs`, etc. This broke clear module boundaries and leaked implementation details.
**Blueprint:** Upgraded `pub mod` to `pub(crate) mod` for all internal modules while keeping `pub mod` only where necessary for public APIs (`parser`, `semantic`, `codegen`, `morphology`) and re-exported features (`highlight`).
**Stability:** Enforced high cohesion and low coupling by ensuring strict encapsulation and clean public interfaces.
**[Breaking The Leak: Encapsulation via pub(crate)]
**Tangle:** Internal modules were needlessly exposed via `pub mod` across various modules. This broke clear module boundaries and leaked implementation details.
**Blueprint:** Upgraded `pub mod` to `pub(crate) mod` for all internal modules while keeping `pub mod` only where necessary for public APIs.

## [Breaking The Leak: Encapsulation via pub(crate)]
**Tangle:** Internal modules were needlessly exposed via `pub mod` across various modules. This broke clear module boundaries and leaked implementation details.
**Blueprint:** Upgraded `pub mod` to `pub(crate) mod` for all internal modules while keeping `pub mod` only where necessary for public APIs.

## [Breaking The Leak: Encapsulation via pub(crate)]
**Tangle:** Internal modules were needlessly exposed via `pub mod` across various modules (`errors/mod.rs`, `morphology/mod.rs`, `parser/mod.rs`, `semantic/mod.rs`, `tools/mod.rs`). This broke clear module boundaries and leaked implementation details.
**Blueprint:** Upgraded `pub mod` to `pub(crate) mod` for all internal modules while keeping `pub mod` only where necessary for public APIs (e.g. `highlight` in `tools/mod.rs` and `lexicon` in `morphology/mod.rs`). Fixed tests to import via the newly defined explicit public interfaces and selectively retained `pub mod` when refactoring deeply nested integration test imports was excessively invasive without providing architectural value.
**Stability:** Enforced high cohesion and low coupling by ensuring strict encapsulation and clean public interfaces.

## [Cleaning The Sprawl: Unused Help Module]
**Tangle:** `src/errors/mod.rs` exported a `pub mod help` containing constants that were never used, polluting the public API.
**Blueprint:** Removed the `help` module entirely to eliminate dead code and enforce a cleaner public interface.
**[Title]** Enforcing the Facade Pattern in src/lib.rs
**Tangle:** The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`). This leaked implementation details and created a sprawling public API, violating the principle of encapsulation and making it difficult for downstream users to know which functions to use.
**Blueprint:** Refactored `src/lib.rs` to change these modules to `pub(crate) mod` (or kept `pub mod` only where explicitly needed by the `glossa` binary or integration tests) and added explicit `pub use` statements for the true public API: `ast::Program`, `codegen::generate_rust`, `parser::parse`, `semantic::{AnalyzedProgram, analyze_program}`. This creates a clean "Facade" that hides messy internal sub-modules while exposing only what the user needs.

**[Breaking The Leak: Encapsulation via pub(crate)]
**Tangle:** Internal modules were needlessly exposed via `pub mod` across various modules (`morphology/mod.rs`, `parser/mod.rs`, `semantic/mod.rs`, `tools/mod.rs`). This broke clear module boundaries and leaked implementation details.
**Blueprint:** Upgraded `pub mod` to `pub(crate) mod` for all internal modules while keeping `pub mod` only where necessary for public APIs. Fixed tests to import via the newly defined explicit public interfaces and selectively retained `pub mod` when refactoring deeply nested integration test imports was excessively invasive without providing architectural value.
**Stability:** Enforced high cohesion and low coupling by ensuring strict encapsulation and clean public interfaces.

**[Enforcing Tool Encapsulation]
**Tangle:** The `src/tools/` directory exposed internal helper modules (`report` and `ui`) as fully public (`pub mod`). This leaked implementation details and created a sprawling public API.
**Blueprint:** Changed the visibility of `report` and `ui` to `pub(crate) mod` to enforce the facade pattern. Fixed a dead code warning on an unused function in `ui` that resulted from the visibility reduction.

**[Tools Facade Refactoring]
**Tangle:** The `src/tools/` directory exposed all of its internal submodules (`alchemist`, `weaver`, `runner`, etc.) publicly as `pub mod`. This leaked internal implementation details and forced external consumers (like `main.rs` and the `tests/` integration directory) to rely on deep nested paths (`glossa::tools::runner::run_file`), breaking encapsulation and increasing structural coupling.
**Blueprint:** Converted all internal tool submodules in `src/tools/mod.rs` to `pub(crate) mod` to enforce a strict boundary. Applied the Facade Pattern by explicitly re-exporting only the necessary functions and structs via a flattened public API (`pub use ...`). Updated `main.rs` and all integration tests to consume the clean, top-level `glossa::tools::*` namespace. Note: Integration tests compile as external crates and cannot access `pub(crate)` or `#[cfg(test)]` items from the library, so internal functions needed for testing were explicitly exported behind the `#[cfg(feature = "nova")]` feature flag rather than using test-specific configuration.
