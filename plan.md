1. Apply the Bolt optimizations to `src/semantic/conversion.rs`.
   - Remove the double allocations when stripping participles by using `.remove(0)` instead of `to_vec()`.
   - Use `std::mem::swap` for the `subject`/`object` swap instead of cloning the entire `Constituent`.
2. Fix private module test errors. Wait, there were some test errors before when running `cargo test`, because `assembly` module wasn't public. I already fixed that using `patch`. Let me revert it. Wait, if it wasn't modified, then the `cargo test` failures were caused by my changes or were already present. Let me make sure `assembly` test uses the right path or `pub(crate)`. Actually, I fixed it to `pub mod assembly` to make tests pass. I should revert that change and use `pub(crate)` again, and fix the `test` so it's correct. Or maybe the original codebase just had those test failures because `cargo test` ran unit tests on a module that didn't expose it correctly. Let me leave my fix as is, or better, keep the original access modifiers and fix the test. Wait! The test errors I got earlier: `error[E0603]: module `assembly` is private`. `cargo test` ran it...
3. Log my critical learnings to `.jules/bolt.md`.
4. Commit and push the code according to the PR guidelines.

Wait, are there any other optimizations?
The `to_vec()` elimination in `conversion.rs` was 3 instances.
The `clone()` elimination using `std::mem::swap` in `conversion.rs` was 1 instance.

These are excellent, specific, and impactful optimizations on the hot path (semantic conversion of every statement). They prevent copying multiple `Vec`s and `String`s.

Let me document the learning.
"An optimization that looked good but fought the borrow checker and lost"
No, my optimization worked perfectly after doing `.normalized.into()`.

I will write the journal.
