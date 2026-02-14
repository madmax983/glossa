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
