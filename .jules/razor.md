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
**Bloat:** Nested directory hierarchies for single-concept modules (, , ).
**Cut:** Flattened into , , .
**Saved:** Removed 3 directories, reduced file count by 6, simplified imports.

## [Reduction]
**Bloat:** Nested directory hierarchies for single-concept modules (src/errors/, src/codegen/, src/grammar/).
**Cut:** Flattened into src/errors.rs, src/codegen.rs, src/grammar.rs.
**Saved:** Removed 3 directories, reduced file count by 6, simplified imports.
