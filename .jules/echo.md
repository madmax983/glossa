**Echo: Fix Missing Errors in Troubleshooting Guide**
**Issue:** The README described helpful errors like `Διπλοῦν ὑποκείμενον` (Double Subject) and `Ἄγνωστον ὄνομα` (Undefined Variable), but they were bypassed and silently evaluated due to flaws in semantic conversion fallbacks and multiple nominative assembly checks.
**Action:**
- Updated `src/semantic/assembly/mod.rs` to ensure double subjects throw `AssemblyError::DoubleSubject` by relying accurately on standard behavior (removing the `is_print_verb` exception that was silently allowing double subjects incorrectly in back-to-back variables).
- Updated `try_print_default` and `classify_expression` in `src/semantic/conversion.rs` to correctly throw `GlossaError::undefined` when evaluating missing variables instead of generating them as fallback `GlossaType::Unknown`.
- Updated failing tests to accurately ensure errors are now thrown and tracked.
