## [Reduction]
**Bloat:** Hardcoded bypasses hiding standard semantic errors. Undefined variable names were ignored by `try_print_default`, `DoubleSubject` checks bypassed `is_print_verb` allowing grammatically broken subjects in print statements, and `check_missing_verb` had a bizarre hardcoded bypass checking specifically `subject.lemma == "ανθρωπος"`.
**Cut:** Removed the silent behavior of `try_print_default` to return `GlossaError::UndefinedName`. Removed `is_print_verb` from the `DoubleSubject` bypass. Completely eliminated the bizarre `is_match_arm` override in `check_missing_verb` causing raw Rustc output errors.
**Saved:** Unnecessary silent compilation failures and compiler logic inconsistencies, resulting in accurate Greek error diagnostics.
