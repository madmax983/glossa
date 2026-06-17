**Testing Semantic Control Flow Guards**
**Learning:** In Glossa, many structural validations (e.g. checking length of clauses, presence of body, etc.) in `semantic/control_flow.rs` and `semantic/declarations.rs` are protected by `GlossaError::semantic` but are unreachable via normal parsing strings because the parser guarantees their structure. Attempting to parse strings to hit these will fail at the PEG grammar level.
**Action:** Construct `Statement` AST nodes manually using `Statement::Regular { ... }` in a unit test to bypass the parser and test the robustness of the semantic layer against unexpected or malformed structures. This ensures that if the grammar rules ever loosen, the semantic analyzer safely rejects the input without panicking.

**Testing Semantic Control Flow Guards**
**Learning:** In Glossa, many structural validations (e.g. checking length of clauses, presence of body, etc.) in `semantic/control_flow.rs` and `semantic/declarations.rs` are protected by `GlossaError::semantic` but are unreachable via normal parsing strings because the parser guarantees their structure. Attempting to parse strings to hit these will fail at the PEG grammar level.
**Action:** Construct `Statement` AST nodes manually using `Statement::Regular { ... }` in a unit test to bypass the parser and test the robustness of the semantic layer against unexpected or malformed structures. This ensures that if the grammar rules ever loosen, the semantic analyzer safely rejects the input without panicking.
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

**Sentry Unit Tests Added for Control Flow Branches**
**Learning:** Found coverage gaps in `src/semantic/control_flow.rs` for `parse_conditional`, `check_else_pattern_in_expression`, and `check_conditional_start`. Realized that coverage unit tests for internal modules `pub(crate)` cannot be placed in the `tests/` directory as integration tests because they will trigger an `E0603: module is private` compilation error.
**Action:** Always embed unit tests for private modules directly within the target file under `#[cfg(test)] mod tests` block to ensure they compile and increase coverage successfully without violating module visibility rules.

**[Semantic Control Flow Panics]
**Learning:** Destructuring deeply nested `AnalyzedStatement` types in integration/unit tests using `if let` with an `else { panic!(...) }` fallback introduces uncovered terminal branches, causing false-positive coverage gaps.
**Action:** Replace `if let` with `let else` patterns or `assert_matches!` to assert structure safely without introducing uncoverable execution branches.

**[Parser Unexpected Rule Defensive Checks]
**Learning:** In the PEG parsing stage, using `match pair.as_rule()` with a generic `_ => Err(ParseError::UnexpectedRule(...))` fallback is good defensive practice, but these branches remain permanently uncovered because the `pest` grammar guarantees input validity before it reaches the AST builder.
**Action:** Craft manual `pest` `Pairs` (often by parsing mismatched rules intentionally) and feed them to the specific AST builder functions inside an embedded `#[cfg(test)] mod tests` block to cover these critical safety guards.
**[Panic Safety in Assembly and Patterns]
**Learning:** Using unreachable!() in fallback match arms acts as a ticking time bomb for structural panics if prior validation assumes impossible conditions.
**Action:** Replace unreachable!() with safe negative-match default returns (e.g., return false; or Ok(false)) and explicitly construct failing unit tests to cover these fallback paths.

**[Codegen Unchecked Indexing and Checked Neg]
**Learning:** Found potential runtime panics in `generate_unary_op` (using `checked_neg().expect(...)`) and in `generate_collection_index` (using `try_from().expect(...)` for index bound checking) in `src/codegen.rs` which were not covered by tests.
**Action:** Wrote isolated unit tests `test_generate_unary_op_neg_checked` and `test_generate_collection_index_bounds_check` to guarantee the panic safeguards execute as intended and generate the appropriate error structures.
**[Report Module Exhaustive Coverage]**
**Learning:** The `AnalyzedExprKind` and `AnalyzedStatement` enums frequently gain new variants (like `CollectionNew`, `TraitImplementation`, `Try`). Match statements with a catch-all `_ => {}` silently ignore these new nodes during traversals, leading to permanently untested code paths and hidden bugs. Using exhaustive matching explicitly forces developers to update traversals when new nodes are added.
**Action:** Replaced `_ => {}` with explicit matches for leaf nodes (`NumberLiteral`, `StringLiteral`, `None`, etc.) in `tell_expr` and `tell_statement` in narrator. Exhaustively instantiated every single AST node variant in `test_tell_expr_all_variants` to ensure the recursive formatting logic actually executes on all possible program structures.
**[Codegen Unchecked Indexing and Checked Neg Runtime Coverage]**
**Learning:** Found potential runtime panics in `generate_unary_op` (using `checked_neg().expect(...)`) and in `generate_collection_index` (using `try_from().expect(...)` for index bound checking) in `src/codegen.rs` which were not covered by integration tests that compile and execute the output. When verifying that the Glossa compiler generates correct runtime panic logic (e.g., for out-of-bounds array access or `unwrap` operations), integration tests must compile the generated Rust code and execute the resulting binary to assert that the runtime panic actually occurs with the expected message, rather than merely checking the generated source string for `.expect()`.
**Action:** Wrote isolated integration tests in `tests/sentry_codegen_perf.rs` to guarantee the panic safeguards execute as intended and generate the appropriate error structures at runtime.
**[Quote Token Spacing in Codegen Tests]**
**Learning:** When asserting on stringified token streams produced by `quote!` in `src/codegen.rs` unit tests (like `test_generate_collection_index_bounds_check`), `quote!` automatically inserts spaces between certain tokens (e.g., `expect ("...")` instead of `expect("...")`). String containment assertions will fail if they don't account for this spacing.
**Action:** Always print or inspect the `.to_string()` output of a `TokenStream` when writing exact string containment assertions to avoid brittle test failures caused by macro formatting.

**[Semantic Patterns Coverage]**
**Learning:** In `src/semantic/patterns.rs`, the logic for parsing unsupported argument types in struct instantiations (`parse_struct_args`), processing `find` without a predicate (`process_find`), and falling back to unstripped suffix matches during genitive comparisons (`extract_comparison_value`) were missing test coverage. Furthermore, when writing tests targeting `process_find` with no predicate, the code actually transforms it to `.find(|_| true)` instead of `.next()`, so asserting against `"next"` causes a test failure.
**Action:** When adding missing edge case coverage, carefully review the inner implementations before constructing the test assertions to match the actual, generated `AnalyzedExpr` nodes instead of assuming idealized shortcuts. Also ensure the setup correctly puts the mock values into the `Scope` using the correct key mapping.

**[Parser Unexpected Rule Defensive Checks]
**Learning:** In the PEG parsing stage, using `match pair.as_rule()` with a generic `_ => Err(ParseError::UnexpectedRule(...))` fallback is good defensive practice, but these branches remain permanently uncovered because the `pest` grammar guarantees input validity before it reaches the AST builder.
**Action:** Craft manual `pest` `Pairs` (often by parsing mismatched rules intentionally) and feed them to the specific AST builder functions inside an embedded `#[cfg(test)] mod tests` block to cover these critical safety guards.

## 2024-06-17 - Codegen Array Bounds Checks
**Learning:** The array index bounds check during code generation panics gracefully via `expect` inside the generated Rust code. We should also verify that negative literals in array indexing during compilation can panic.
**Action:** Wrote `sentry_codegen_bounds.rs` to verify that `expect()` and panic are properly injected into output code.
