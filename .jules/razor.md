## [Reduction]
**Bloat:** `src/semantic/disambiguation.rs` containing purely morphological logic.
**Cut:** Moved to `src/morphology/disambiguation.rs` and re-exported from `crate::morphology`.
**Saved:** Removed "Layer Lasagna" dependency (semantic -> disambiguation -> morphology). Now semantic -> morphology.

## [Fix]
**Issue:** `tests/havoc_stack_overflow.rs` used deprecated `build_ast` API.
**Fix:** Switched to `glossa::parser::parse`.
**Saved:** Fixed a test failure and removed usage of potentially dead API.

## [Reduction]
**Bloat:** `convert_expr_to_analyzed` in `src/semantic/expressions.rs` duplicated logic from `analyze_argument_expr` and silently returned `0` for unknown expressions.
**Cut:** Merged logic into `analyze_argument_expr` and deleted `convert_expr_to_analyzed` and `convert_array_elements`.
**Saved:** Removed dangerous silent failure bug, reduced code duplication (~50 lines removed), enforced error propagation.

## [Reduction]
**Bloat:** `ExecutionMode`, `AnalyzedWord`, `LambdaKind`, `Expr::Lambda` were dead or duplicated abstractions.
**Cut:** Deleted them.
**Saved:** Simplified AST and type system. Removed zombie code that wasn't used by the parser or semantic analyzer.

## [Reduction]
**Bloat:** `src/ir` module (HIR) was a mirror of `AnalyzedProgram` with English names but identical structure.
**Cut:** Deleted `src/ir` (~600 lines). Updated `codegen` to consume `AnalyzedProgram` directly.
**Saved:** Removed an entire compiler pass and module. Flattened architecture: Parser -> Semantic -> Codegen.

## [Reduction]
**Bloat:** Duplicated logic for extracting comparison values and creating predicates in iterator patterns (`filter`, `any`, `all`, `find`).
**Cut:** Extracted `extract_comparison_value` and `create_comparison_predicate` helper functions.
**Saved:** ~150 lines of duplicated code. Reduced cognitive load for understanding iterator pattern logic.

## [Reduction]
**Bloat:** `MethodSignature`, `DefaultMethod`, `ImplMethod`, and redundant `TraitDef` fields in `src/semantic/model.rs`.
**Cut:** Consolidated into `TraitDef` using `Vec<AnalyzedTraitMethod>` and simplified `TraitImpl`. Deleted redundant structs.
**Saved:** Reduced code duplication and complexity in semantic model and declarations. ~50 lines removed.

## [Reduction]
**Bloat:** Duplicate logic for struct instantiation in `src/semantic/conversion.rs` (broken, handled only literals) and `src/semantic/patterns.rs` (working, handled variables).
**Cut:** Deleted `classify_struct_instantiation` from `conversion.rs` and consolidated `detect_collection_type` into `src/semantic/types.rs`.
**Saved:** Removed ~100 lines of duplicate/broken code and fixed a bug preventing variable arguments in constructors.

## [Reduction]
**Bloat:** `AnalyzedTraitMethod` and `AnalyzedImplMethod` in `src/semantic/model.rs`.
**Cut:** Merged into a unified `AnalyzedMethod` struct. Removed redundant `is_default` field.
**Saved:** Reduced code duplication in semantic model and codegen (~60 lines removed/simplified). Unified method representation.

## [Reduction]
**Bloat:** Duplicated "statement dispatch" logic in `analyze_program`, `analyze_trait_definition`, `analyze_trait_impl`, and `parse_function_definition`.
**Cut:** Centralized in `analyze_statement` helper in `src/semantic/mod.rs`.
**Saved:** Removed duplicate `if/else` chains (~40 lines). Enabled consistent block (`{...}`) support across all contexts.

## [Fix]
**Issue:** `parse_function_definition` used `Scope::new()` which created a detached scope, preventing access to global types/traits.
**Fix:** Switched to `scope.enter_scope()`.
**Saved:** Fixed bug where functions couldn't see global definitions. Corrected scope inheritance.
