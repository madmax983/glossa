1. **Update `extract_object_fallback` to enforce `UndefinedName` error.**
   - Modify `src/semantic/conversion.rs` at `extract_object_fallback`.
   - When returning a variable expression for an object, look it up in the `scope`.
   - If it doesn't exist, return `Err(GlossaError::undefined(&obj.lemma))`.

2. **Update `extract_subject_fallback` (or where subjects are used as values) to enforce `UndefinedName`.**
   - The memory states: `extract_value` must include an `extract_subject_fallback` alongside `extract_object_fallback` to catch scrutinees (like `χ` in `κατὰ χ`).
   - Add `extract_subject_fallback` in `src/semantic/conversion.rs`.
   - If the subject is used as a value, enforce that it's defined in the scope.

3. **Update `try_print_default` to enforce `UndefinedName` error.**
   - In `src/semantic/conversion.rs`, when printing variables (either subject or object), check if they are defined in the scope using `scope.lookup()`.
   - Instead of silently skipping them, return `Err(GlossaError::undefined(&subj.lemma))` or `&obj.lemma`.

4. **Verify the changes using `havoc_issue_echo.rs`.**
   - Confirm that the `UndefinedName` errors are properly propagated when using an undefined variable.
   - Confirm that a MissingVerb or DoubleSubject error occurs in `havoc_issue_echo.rs` when needed. (Note: MissingVerb currently causes codegen panics. I'll make sure `DoubleSubject` check works during `finalize()` in `assembly.rs` by updating `classify_expression` to bypass irregular cases as mentioned in the memory).

5. **Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Run `pre_commit_instructions` tool.

6. **Submit changes**
   - Commit the branch using the provided conventions.
