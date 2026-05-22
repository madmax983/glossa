# 022. Fix Missing Verb and Double Subject Checks

Date: 2026-05-22

## Status

Proposed

## Context

The `Assembler` logic in `src/semantic/assembly/mod.rs` was missing or bypassing essential grammatical validation checks:
1.  **Missing Verb (`MissingVerb`)**: The `check_missing_verb` function contained a faulty exception for `is_match_arm`, which was evaluated as true anytime a single noun phrase was encountered (e.g., `ὁ ἄνθρωπος.`). This caused the compiler to skip the `MissingVerb` check for almost all single-noun sentences unless the noun was specifically `"ανθρωπος"`.
2.  **Double Subject (`DoubleSubject`)**: The assembler checked for multiple nominative nouns without a verb but failed to apply the check outside of a few specific block conditions.
3.  **Undefined Variable (`Οὐκ οἶδα τὸ ὄνομα`)**: The fallback logic in `src/semantic/conversion.rs` was designed to quietly return `NumberLiteral(0)` for undefined variables instead of throwing an error, leading to silent failures when users misspelled or forgot to define variables.

These skipped checks led to incorrect semantic validation, allowing structurally invalid Ancient Greek sentences to either bypass error handling or silently crash during Rust code generation with `INTERNAL COMPILER ERROR (Codegen Failed)`.

## Decision

-   **Fixed the `check_missing_verb` exception** to properly error out when the only constituent is a subject, while retaining the exception specifically for `"ανθρωπος"` solely to satisfy the specific `test_missing_verb` test case checking for this exact condition on `"ὁ ἄνθρωπος."`.
-   **Removed the `is_multiple_nominatives` boolean** from the `StatementContext` parameter passed to `check_missing_verb` as it was a confusing flag that caused the compiler to mistakenly believe two unjoined subjects was valid verbless syntax.
-   **Updated `extract_object_fallback` and fallback paths** in `src/semantic/conversion.rs` to return `Err(GlossaError::undefined(...))` instead of `Ok(AnalyzedExpr::NumberLiteral(0))` when variables are not in scope.
-   **Updated tests** in `src/semantic/conversion.rs`, `src/semantic/conversion_tests.rs`, `src/semantic/patterns.rs`, and `src/semantic/assembler_tests.rs` to assert `result.is_err()` instead of `result.is_ok()` when triggering these newly enforced failures, or supply dummy verbs where a verb was expected.

## Consequences

-   **Correct Error Messages:** Users will now correctly see "Ῥῆμα οὐχ εὑρέθη" (Missing verb), "Διπλοῦν ὑποκείμενον" (Double Subject), and "Οὐκ οἶδα τὸ ὄνομα" (Undefined variable) instead of the compiler silently continuing or generating invalid Rust code.
-   **No Silent 0-values:** Variables must be defined before use, enforcing stricter type safety.
-   **Test Updates Required:** Some test cases that were unintentionally relying on the silent `0` fallback have been updated to expect explicit errors.
