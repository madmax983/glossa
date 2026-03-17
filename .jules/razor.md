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
**Bloat:** `src/ast/` and `src/codegen/` directories were merely wrapper folders around `mod.rs`.
**Cut:** Moved `mod.rs` to `src/ast.rs` and `src/codegen.rs` and deleted the wrapper directories.
**Saved:** 2 folders, reduced directory depth.

## [Reduction]
**Bloat:** `src/semantic/traits.rs` defined a `StatementAnalyzer` trait that was only ever implemented once by `Analyzer` in `src/semantic/analyzer.rs`.
**Cut:** Deleted the trait and file entirely, calling `Analyzer::analyze` concretely.
**Saved:** 1 file, reduced indirection, simplified function signatures in `control_flow.rs` and `declarations.rs`.

## [Reduction]
**Bloat:** Trivial string-formatting helper functions (`undefined_variable`, `immutable_assignment`, `gender_mismatch`, `number_mismatch`, `case_mismatch`) in `src/errors.rs` added unnecessary abstraction and boilerplate. Four of these functions were actually unused dead code.
**Cut:** Inlined the single active usage (`immutable_assignment`) directly at its call site and deleted all 5 functions from `src/errors.rs` entirely.
**Saved:** Removed 5 trivial functions, 4 of which were dead code, and their associated tests, reducing line count by ~70 lines and improving clarity.

## [Reduction]
**Bloat:** `src/experimental/` directory containing only an `interpreter.rs` tool.
**Cut:** Moved `interpreter.rs` to `src/tools/` and deleted the `experimental` module entirely.
**Saved:** 1 directory, reduced module depth, strictly enforced production quality for all files in `src/`.

## [Reduction]
**Bloat:** `Analyzer` struct in `src/semantic/analyzer.rs` was completely empty with no fields, used purely as a namespace for the `analyze` method that was passed around to other modules.
**Cut:** Deleted the `Analyzer` struct and converted `Analyzer::analyze` into a standalone `analyze_statement` function. Updated calling signatures across `control_flow.rs` and `declarations.rs` to remove the unnecessary `analyzer: &mut Analyzer` parameter.
**Saved:** Removed empty struct instantiation, cleaned up ~20 function signatures, improved modular cohesion and flattened architectural layers.

## [Reduction]
**Bloat:** `MAX_*` depth/limit constants scattered across `src/semantic/assembly/model.rs` and `src/semantic/control_flow.rs`, creating architectural duplication and inconsistent definition points.
**Cut:** Centralized all compiler depth limit constants into `src/limits.rs`.
**Saved:** Centralized logic (Single Source of Truth) and improved transparency for architectural constraints.

## [De-abstract StatementAnalyzer Trait]
**Bloat:** The `StatementAnalyzer` trait was introduced to resolve a circular dependency between `semantic::analyzer`, `semantic::control_flow`, and `semantic::declarations`. However, since these modules all live within the same crate (`semantic`), Rust permits circular module dependencies natively via free functions, rendering the trait abstraction unnecessary. Additionally, `StatementAnalyzer` was a single-implementation trait only implemented by the empty struct `SemanticAnalyzer`.
**Cut:** Deleted the `StatementAnalyzer` trait and the `SemanticAnalyzer` struct entirely. Replaced trait method dispatch across submodules with direct calls to `crate::semantic::analyzer::analyze_statement`.
**Saved:** Eliminated trait boilerplate, reduced the number of files by deleting `src/semantic/traits.rs`, and removed empty struct allocations, significantly reducing cognitive overhead by flattening the semantic analysis architecture.
## [Reduction]
**Bloat:** `src/semantic/assembly/` directory contained `mod.rs` and `model.rs`. `model.rs` contained DTOs tightly coupled to the assembler.
**Cut:** Merged `model.rs` into `mod.rs` and renamed it to `src/semantic/assembly.rs`, deleting the `assembly/` folder.
**Saved:** 1 directory, 1 file, reduced module indirection.
## [Reduction]
**Bloat:** The `QuantifierFlags` struct and its `from` method, which only held and passed around two boolean values (`is_any`, `is_all`).
**Cut:** Replaced `QuantifierFlags` with a simple function `get_quantifiers` that returns a `(bool, bool)` tuple, passing the booleans directly to helper functions.
**Saved:** Unnecessary struct definition, struct instantiation, and method implementation (~10 lines of code and cognitive load).
