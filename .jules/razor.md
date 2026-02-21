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
**Bloat:** `src/experimental/numerals.rs` was a fully functional module inside a misleading "experimental" folder.
**Cut:** Moved to `src/parser/numerals.rs` and deleted `src/experimental`.
**Saved:** 1 folder, clearer project structure.

## [Reduction]
**Bloat:** Ad-hoc helper functions (`case_name`, `gender_name`, `number_name`) in `src/errors/messages.rs` for string conversion.
**Cut:** Implemented `std::fmt::Display` for `Case`, `Gender`, and `Number` enums in `src/morphology/mod.rs`.
**Saved:** Removed 3 helper functions, enforced standard Rust traits.

## [Reduction]
**Bloat:** `src/semantic/resolver.rs` maintained 4 separate HashMaps (`bindings`, `functions`, `types`, `traits`) and duplicate code for each.
**Cut:** Unified all symbols into a single `HashMap<SmolStr, Symbol>` using a `Symbol` enum.
**Saved:** Reduced code duplication, simplified lookup logic, enforced single namespace clarity.

## [Reduction]
**Bloat:** `src/semantic/types.rs` contained an `Ownership` enum ("Move/Borrow/Copy") used only in self-tests, likely speculative future-proofing.
**Cut:** Deleted the `Ownership` enum and its tests.
**Saved:** Removed dead code (~30 lines).

## [Reduction]
**Bloat:** `src/errors/mod.rs` defined `TypeError` and `IoError` variants that were never used by the compiler.
**Cut:** Deleted these error variants and their helper functions.
**Saved:** Removed dead code and reduced error surface area.

## [Reduction]
**Bloat:** `src/cli.rs` was a dead duplicate of `src/tools/cli.rs`.
**Cut:** Deleted `src/cli.rs`.
**Saved:** 1 file, eliminated confusion.

## [Reduction]
**Bloat:** `src/ast/` directory contained only `mod.rs` and `nodes.rs`.
**Cut:** Flattened to `src/ast.rs`.
**Saved:** 1 folder, 1 file, reduced module depth.

## [Reduction]
**Bloat:** `src/codegen/` directory split logic across 4 files unnecessarily.
**Cut:** Flattened to `src/codegen.rs`.
**Saved:** 1 folder, 3 files, simplified imports.

## [Reduction]
**Bloat:** `src/grammar/` directory wrapper for a single pest file.
**Cut:** Flattened to `src/grammar.rs` and `src/grammar.pest`.
**Saved:** 1 folder, reduced module depth.

## [Reduction]
**Bloat:** `src/parser/` directory logic split between `mod.rs` and `builder.rs`.
**Cut:** Merged into `src/parser.rs` (keeping `numerals.rs` as submodule).
**Saved:** 1 file, improved cohesion.

## [Reduction]
**Bloat:** `src/experimental/bard.rs` was a fully functional tool hidden in an "experimental" module.
**Cut:** Promoted to `src/tools/narrator.rs` and deleted `src/experimental`.
**Saved:** 1 folder, 2 files, clearer project structure.

## [Reduction]
**Bloat:** `src/errors/` folder structure for simple error types.
**Cut:** Flattened into `src/errors.rs`.
**Saved:** 2 files, 1 folder, clearer imports.

## [Reduction]
**Bloat:** `src/morphology/matcher.rs` single function abstraction.
**Cut:** Moved `match_suffix` to `src/morphology/mod.rs`.
**Saved:** 1 file, reduced indirection.

## [Reduction]
**Bloat:** `src/parser/recursion.rs` single check function.
**Cut:** Moved `check_recursion_depth` to `src/parser.rs`.
**Saved:** 1 file.

## [Reduction]
**Bloat:** `src/semantic/assembly_model.rs` DTOs separate from logic.
**Cut:** Merged into `src/semantic/assembler.rs`.
**Saved:** 1 file, tighter cohesion.

## [Reduction]
**Bloat:** Unused fields in `Constituent`, `VerbConstituent`, `ParticipleConstituent` (case, original, tense, voice).
**Cut:** Removed fields and simplified struct construction.
**Saved:** ~20 lines, reduced memory footprint, YAGNI compliance.

## [Reduction]
**Bloat:** `src/tools/cache.rs` and `src/tools/cli.rs` small modules.
**Cut:** Merged `Cache` into `src/tools/runner.rs` and `Cli` into `src/tools/mod.rs`.
**Saved:** 2 files.

## [Reduction]
**Bloat:** Integration test `tests/regression_operator_drop.rs` testing internal `pub(crate)` items.
**Cut:** Moved to unit test in `src/semantic/expressions.rs`.
**Saved:** 1 file, removed need for `pub` visibility leaks.
