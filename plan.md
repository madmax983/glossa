1. **Understand the problem**: The issue states that the compiler incorrectly allows missing verbs, double subjects, and undefined variables.
2. **Missing Verb (INTERNAL COMPILER ERROR)**:
   - Fix `src/semantic/assembly/mod.rs` to correctly throw `AssemblyError::MissingVerb` instead of returning `Ok(())` on single-noun statements, removing a flawed check on the pseudo-property `is_match_arm`.
   - Update `test_missing_verb` in `src/semantic/assembler_tests.rs` to actually assert an error. (Actually wait, the codebase had `is_match_arm` flag. The PR fixes `test_missing_verb` correctly).
3. **Double Subject (silent compilation)**:
   - Remove the `is_multiple_nominatives` boolean bypass which wrongly excused double subjects without verbs. Now they produce "Διπλοῦν ὑποκείμενον" (Double Subject).
4. **Undefined Variable (silent fallback to 0)**:
   - In `src/semantic/conversion.rs`, change the fallback paths that returned `NumberLiteral(0)` to instead return `Err(GlossaError::undefined(name))`.
5. **Update tests**:
   - Update `src/semantic/conversion_tests.rs` to assert `is_err()` instead of `is_ok()` with fallback literals.
   - Inject dummy verbs into `AssembledStatement` instances in `src/semantic/patterns.rs` test cases to prevent them from failing the new strict `MissingVerb` check.
   - Make sure all `cargo test` pass.
6. **Complete pre commit steps**
   - Complete pre commit steps to make sure proper testing, verifications, reviews and reflections are done.
7. **Submit the change.**
   - Once all tests pass, I will submit the change with a descriptive commit message and updated ADR document.
