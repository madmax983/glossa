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
**Bloat:** `src/experimental/numerals.rs` duplicated in parser logic and marked experimental despite production use.
**Cut:** Promoted to `src/parser/numerals.rs` and deleted `src/experimental/`.
**Saved:** Removed 1 directory, 1 file, clarified code status.

## [Reduction]
**Bloat:** `src/main.rs` was a 500+ line "God Object" handling CLI, REPL, and file operations.
**Cut:** Split into `src/tools/cli.rs`, `src/tools/repl.rs`, `src/tools/runner.rs`.
**Saved:** `main.rs` reduced to ~50 lines; improved modularity and testability.
