## [Morphological Ambiguity]
**Learning:** `analyze_noun` relies on a fixed check order (Third -> Second -> First), which causes `First Declension Alpha` nouns (ending in -α) to be misidentified as `Second Declension Neuter` plurals (also ending in -α) if they don't have a lexicon entry. `analyze_noun_all` correctly identifies both but requires the caller to disambiguate.
**Action:** Use `analyze_noun_all` when dealing with ambiguous endings or ensure the lexicon is populated. Tests for ambiguous words must verify the *presence* of the correct analysis, not just the first one.

## [Panic Safety in Morphology Analysis]
**Learning:** `analyze_all` in `src/morphology/mod.rs` was sorting analyses by confidence using `unwrap()` on `partial_cmp`. If `confidence` is `NaN`, this causes a panic.
**Action:** Replaced `unwrap()` with `unwrap_or(std::cmp::Ordering::Equal)` to handle NaN safely. Always verify `partial_cmp` on floats is handled safely.

## [State Modification in Condition Evaluation]
**Learning:** In `src/semantic/assembler.rs`, `check_method_verbs` was popping from `pending_literals` inside a complex `if` condition using `&&`. If subsequent conditions (like `pending_subject` check) failed, the literal was already consumed and lost, causing a logic bug.
**Action:** Always verify all preconditions (using `peek` or `last()`) before performing state-modifying operations (like `pop()`), especially in `if` conditions.

## [Grammar] Person Agreement Check
**Learning:** The assembler was checking Number agreement but missing Person agreement, allowing incorrect sentences like "I (1st) says (3rd)".
**Action:** Added `person` field to `Constituent` and updated `finalize` to check `subj_person != verb_person`. Nouns default to 3rd person.

## [Silent Token Swallowing in Special Checks]
**Learning:** Functions like `check_method_verbs` and `check_special_properties` were returning `true` (indicating "handled") even when they failed to match the full pattern (e.g., missing subject), causing tokens like "split" or "length" to be silently ignored instead of falling back to normal verb/noun processing.
**Action:** Ensure that special handling functions only return `true` when they *successfully* handle the token. If prerequisites aren't met, return `false` to allow fallback to standard processing.

## [Resource Exhaustion (DoS) in Assembler]
**Learning:** The `Assembler` was using unbounded `Vec::push()` for adjectives, literals, and other constituents. This allowed malicious input (e.g., infinite adjectives) to cause OOM panics or Denial of Service.
**Action:** Implemented strict resource limits (`MAX_ADJECTIVES`, etc.) and a new `LimitExceeded` error. Always validate collection sizes before pushing, especially in assemblers/parsers exposed to user input.

## [Silent Token Swallowing in Assembler Fallback]
**Learning:** In `src/semantic/assembler.rs`, the fallback logic for unknown case tokens was silently swallowing tokens if the object slot was already full. It would attempt to set `state.object`, find it occupied, and then do nothing (returning `Ok(())`), leading to data loss.
**Action:** Changed fallback logic to return `Err(AssemblyError::DoubleObject)` when the object slot is full, ensuring no tokens are silently ignored.

## [Constraint] Empty Stem in Suffix Matching
**Learning:** `match_suffix` in `src/morphology/matcher.rs` explicitly skips matches if the resulting stem is empty. This prevents spurious matches (e.g., word "o" matching suffix "-o"), but means words that coincide with their suffix (e.g., article "o") must be handled by the lexicon or explicit checks, not morphological rules.
**Action:** When implementing morphology rules, assume `match_suffix` requires `word.len() > suffix.len()`. Add explicit test cases for this constraint to document it as intended behavior.

## [Safe Slice Pattern Matching]
**Learning:** Checking the length of a vector and then safely using `.last().unwrap()` is considered statically safe but violates the zero-panic-risk paradigm. Simply replacing it with `if let Some` can lead to clippy `collapsible_if` warnings and redundant nesting.
**Action:** When parsing structured string components (like space-separated test outputs), prefer safe slice pattern matching (e.g., `if let [_, name, .., status_str] = parts.as_slice()`) instead of redundant length checks and `.last().unwrap()` indexing to maintain strict panic-free safety and concise code.

## [Error Message Consistency]
**Learning:** `AGENTS.md` explicitly calls for Greek error messages in the compiler, but random unprompted strings might confuse other agents/reviewers.
**Action:** Ensure error context provided via `ok_or_else()` uses standard English unless explicitly writing a user-facing compiler diagnostic intended to be translated.

**2025-03-05 - Avoid unreachable!() in Pattern Matching Fallbacks**
**Learning:** `unreachable!()` inside complex logic like the `process_participles` pattern matcher in `src/semantic/patterns.rs` acts as a ticking time bomb. Even if the expected conditions are validated elsewhere, unexpected invalid configurations (like `BinaryOp::Sub` in a fold pattern) could sneak in during earlier assembly phases and cause panic instead of graceful failures.
**Action:** Replace `unreachable!()` with safe negative-match default returns (e.g., `return (false, false);` or `None`), and explicitly construct a failing unit test under `#[cfg(test)] mod tests` to cover this exact structural configuration.

**Testing UI Status that consumes self**
**Learning:** `Status::success` and `Status::error` take ownership of `self` (`mut self`), which means you cannot call them and then attempt to use the `Status` struct again in the same scope, or even call them twice, because the value is moved. Testing `.update()` on an inactive status must be done by mutating the `active` flag directly if the API doesn't allow re-use after completion.
**Action:** When testing UI structs that implement consuming builder/completion patterns, either construct fresh instances per test phase or manipulate internal state directly if the goal is to test protective early returns.

**Testing Output Parsers with Edge Case Strings**
**Learning:** When testing output parsers like `extract_failures` that expect structured text (like `rustc --test` output), using strings that partially match the expected structure but lack content (e.g., just `failures:\n\n\n\n`) is an effective way to trigger and cover edge case bounds checks.
**Action:** Always include table-driven or isolated tests for text parsers that feed them "almost correct" but structurally empty or invalid data to hit early returns and empty-state branches.
**2025-02-14 - [Report Module Exhaustive Coverage]**\n**Learning:** The `AnalyzedExprKind` and `AnalyzedStatement` enums frequently gain new variants (like `CollectionNew`, `TraitImplementation`, `Try`). Match statements with a catch-all `_ => {}` silently ignore these new nodes during traversals, leading to permanently untested code paths and hidden bugs. Using exhaustive matching explicitly forces developers to update traversals when new nodes are added.\n**Action:** Replaced `_ => {}` with explicit matches for leaf nodes (`NumberLiteral`, `StringLiteral`, `None`, etc.) in `visit_expr`. Exhaustively instantiated every single AST node variant in `test_program_stats_coverage` to ensure the recursive `ProgramStats` logic actually executes on all possible program structures, boosting coverage from 89% to >97%.
**Sentry Unit Tests Added for Control Flow Branches**
**Learning:** Found coverage gaps in `src/semantic/control_flow.rs` for `ἕως` and `ἀπὸ` parsing logic. Realized that to properly test CLI integrations under `cargo llvm-cov`, the binary path resolution fallback logic must explicitly check the `target/llvm-cov-target/debug/` directory as `cargo test --lib` does not provide `CARGO_BIN_EXE_glossa`.
**Action:** Wrote specific manual AST-driven unit tests for control flow parsing, and updated `CARGO_BIN_EXE_glossa` fallback paths in `tools/runner.rs` and `tools/tester.rs` to support both standard testing and code coverage collection without crashing.
