# Razor's Journal

## [Reduction]
**Bloat:** `analyze_participle` allocates and sorts a vector of patterns on every call.
**Cut:** Use `LazyLock` to pre-compute the sorted patterns once.
**Saved:** N/A lines (performance fix), but cleaner logic.

## [Reduction]
**Bloat:** `src/morphology/case.rs` is a small file containing only enums.
**Cut:** Move enums to `src/morphology/mod.rs` and delete `case.rs`.
**Saved:** 1 file, reduced module depth.
**Bloat:** Wrapper function `parse_clause_as_mini_statement` in `src/semantic/control_flow.rs` used only locally.
**Cut:** Inlined into 4 call sites.
**Saved:** Removed unnecessary abstraction (~10 lines).

## [Reduction]
**Bloat:** `src/semantic/assembled.rs` containing DTOs (`AssembledStatement`, `Constituent`) tightly coupled to `src/semantic/assembler.rs`.
**Cut:** Merged `assembled.rs` into `assembler.rs` and deleted the file.
**Saved:** Removed 1 file, reduced module indirection, improved cohesion.

## [Reduction]
**Bloat:** `src/tools/highlight.rs` isolated in its own directory.
**Cut:** Moved to `src/highlight.rs`.
**Saved:** 1 directory, reduced nesting.

## [Reduction]
**Bloat:** `src/ast` directory with `mod.rs` and `nodes.rs`.
**Cut:** Flattened to `src/ast.rs`.
**Saved:** 1 directory, 1 file, reduced nesting.

## [Reduction]
**Bloat:** `src/errors` directory with multiple small files.
**Cut:** Flattened to `src/errors.rs`.
**Saved:** 1 directory, 2 files, reduced nesting.

## [Reduction]
**Bloat:** Separated `src/grammar` (pest) and `src/experimental` (numerals) from `src/parser`.
**Cut:** Consolidated all parsing logic into `src/parser` (moving files, creating submodules).
**Saved:** 2 directories, improved cohesion (all parsing in one place).
