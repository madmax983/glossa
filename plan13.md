Wait, in my original change, I replaced `if let crate::ast::Statement::Regular { .. } = stmt { ... } else { match stmt ... }`
with `match stmt { crate::ast::Statement::Regular { .. } => { ... } stmt_other => { match stmt_other { ... } } }`.

Because I modified those lines, `cargo llvm-cov` expects me to have covered them!
If I just reverted the `if let` block to EXACTLY what it was before, I would only be modifying the `Err(e)` line.
So the diff would ONLY contain the `Err(e)` line.
And the `Err(e)` line IS hit by my manual error test:
`let source = "ὁ ἄνθρωπος λέγει γράφει."; // Double verb triggers an AssemblyError directly`

If I ONLY modify the `Err(e)` line and `output.contains("\u{3a3}... (Error)")` in the test, then MY ENTIRE DIFF is 100% covered!

Why did I change the `if let` into a `match`?
Because earlier, `_ => "Unknown"` was flagged by `cargo llvm-cov` as uncovered. But wait, `cargo llvm-cov` on PR diffs only flags lines THAT ARE PART OF THE DIFF.
If `_ => "Unknown"` was NOT part of my diff, Codecov would NOT count it against me for the PATCH diff coverage!
My PR's PATCH target is what failed. Patch target only looks at the lines I changed.
If I changed `_ => "Unknown"` to `unreachable!()`, then I created a new line in the patch that was uncovered.

Let's revert my `src/tools/mosaic.rs` to its original state from `efb71e0` and ONLY apply the localized error message change + test.
