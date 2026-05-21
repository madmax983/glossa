## [Reduction]
**Bloat:** Undefined variables silently evaluating to 0 and preventing expected `DoubleSubject` and `MissingVerb` errors from bubbling up to the user by short-circuiting to silent fallbacks.
**Cut:** Eliminated silent `0` evaluation fallbacks in `extract_value`, `classify_expression`, and `try_print_default` in `src/semantic/conversion.rs`, and modified `analyze_word` in `src/semantic/expressions.rs` to propagate `UndefinedName`. Removed the exclusion of print verbs in `src/semantic/assembly/mod.rs` that prevented `DoubleSubject` errors.
**Saved:** Reduced developer cognitive load by making errors explicit, aligning actual compiler behavior with the troubleshooting documentation.
