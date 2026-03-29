Line 118 is `crate::ast::Statement::TestDeclaration(_) => "Test Declaration",`
Wait! `cargo llvm-cov` says `/app/src/tools/mosaic.rs: 119` is uncovered!
What is on line 119?
```rust
118:                    crate::ast::Statement::TestDeclaration(_) => "Test Declaration",
119:                    crate::ast::Statement::Regular { .. } => unreachable!(),
```
YES!!!! Line 119 is the `Regular { .. } => unreachable!()`!
NOT `TestDeclaration`!
I removed `unreachable!()` and put `""` before, and it said line 119 was uncovered!
Then I put `unreachable!()` back, and it still says line 119 is uncovered?
Wait, if it's `unreachable!()`, `llvm-cov` normally ignores it. But wait, `unreachable!()` expands to `panic!("internal error: entered unreachable code")`.
If a line has a macro expansion that is NOT executed, `llvm-cov` marks it as uncovered IF the user doesn't exclude it!
Wait! The GitHub Action failed check is NOT about `unreachable!()`!
Let's see the Github CI error!
`81.81% of diff hit (target 93.51%)`
The diff hit includes `unreachable!()` because it's a line I modified!
Ah! By replacing `_ => "Unknown"` with `crate::ast::Statement::Regular { .. } => unreachable!()`, I added a line to the diff! And this line is never hit (because it's unreachable), so its coverage is 0%!
This drops the diff coverage!

To fix this, I MUST NOT add a new line that is not hit.
How do I make the `match` exhaustive without adding a missed line?
Just leave `_ => ""` on line 119? But I DID THAT and it was still marked as uncovered because `_ => ""` is NOT hit!
If I add a line `_ => ""` and it's in the diff, and it's not hit, the diff coverage drops!

How to NOT add a line to the diff?
Before my PR, line 119 was: `_ => "Unknown",`.
If I leave it as `_ => "Unknown",` then I didn't modify it.
Wait, my original plan modified `_ => "Unknown",` to `crate::ast::Statement::Regular { .. } => unreachable!(),` because I thought `Unknown` was wrong or I was trying to fix coverage.
Let's restore the `match` EXACTLY as it was before, except for the `Err(e)` part which is what I came here to fix!
