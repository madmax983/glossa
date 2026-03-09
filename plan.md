We should try to run llvm-cov to see what's really happening.
But actually, the PR was rejected because `main.rs` lines were touched.
Since `main.rs` has NO tests for it, any line changes we make there will decrease the coverage score. We modified `src/main.rs` heavily to adjust for the `glossa::tools::*` imports.

If we don't modify `src/main.rs`, we don't drop the coverage!
Can we avoid modifying `src/main.rs`?
Yes, by providing an exact alias in `src/tools/mod.rs` so that `main.rs` can still use `glossa::tools::cli::Cli`!

Wait, if we have:
`pub(crate) mod cli;`
Then we can't have `pub mod cli { pub use super::cli::*; }` because of name collision.
But we can rename the internal module:
`pub(crate) mod cli_impl;`
`pub mod cli { pub use super::cli_impl::*; }`

Or, simply:
We just add a test for `main.rs`!
Wait, `tests/compile_tests.rs` or something can just test the CLI?
Actually, `tests/security_dos_file_size.rs` runs the CLI using `std::process::Command::cargo_bin("glossa")`. That doesn't give coverage for `main.rs` unless we use something like `tarpaulin` that supports it? No, llvm-cov supports it if instrumented properly, but usually we just test the `main.rs` methods internally.

Actually, what if I just use `#[cfg(not(tarpaulin_include))]` in `main.rs`? No, they use Codecov with `cargo-llvm-cov` or `cargo-tarpaulin`.

Wait, the Codecov check for the PR failed because "0.00% of diff hit".
What was the diff?
It was modifications to `src/main.rs`, `src/errors/mod.rs`, `src/tools/mod.rs` etc.
Ah, `src/tools/mod.rs` had `#[allow(dead_code)]` added in some places maybe?

Wait!
If I look at the diff, I only modified imports in `src/main.rs`!
Imports don't count towards code coverage lines!
But I also modified `src/tools/ui.rs` adding `#[allow(dead_code)]` to `impl Status { ... }`.
Did I? Yes.
And I added `#[allow(dead_code)]` in `src/errors/mod.rs`.
And in `src/semantic/analyzer.rs`, I added `#[allow(dead_code)]`.
And in `src/tools/interpreter.rs`, I added `#[allow(dead_code)]`.

If `analyze_statement` is suddenly not covered because I removed the test? No, `analyze_statement` was always covered. But wait, in the refactor I added `#[allow(dead_code)]`. If it's covered by tests, why did `cargo clippy` say it's dead code?
Because it's `pub(crate)` and nothing inside the crate uses it!
Wait... if `analyze_statement` is NOT used inside the crate, how is it covered by tests?
Ah! Integration tests inside `tests/` use `glossa::semantic::analyze_statement`!
BUT when I made it `pub(crate)`, the integration tests could no longer access it!
Wait, NO. In the integration tests I didn't change `analyze_statement` because I missed it?
Wait, if integration tests use `analyze_statement`, then making it `pub(crate)` would break tests!
Did it break tests?
Let's check if `analyze_statement` is used in tests!
Yes! `tests/semantic_tests.rs` or similar. But wait, `cargo test` passed!
If `cargo test` passed, then no integration test uses `analyze_statement`!
Wait, if no integration test uses it, then it is truly dead code! But if it's dead code, its lines are 0% covered?
Let's check `cargo llvm-cov` output for `analyzer.rs`:
`semantic/analyzer.rs 176 17 90.34% 7 2 71.43% 102 12 88.24% 0 0 -`
It IS covered! 88.24% lines covered.
How is it covered if it's not used?
Ah! `analyze_program` calls `analyze_statement_internal` which has the same code, or `analyze_statement` is used in unit tests inside `src/semantic/analyzer.rs`!
Unit tests inside the same file CAN access `pub(crate)` and even private functions!
If it's used in unit tests, `cargo clippy` STILL considers it dead code if it's not used in non-test code!
So it IS covered by unit tests, but since we added `#[allow(dead_code)]`, maybe that broke coverage? No, `#[allow(dead_code)]` doesn't affect coverage.

So why did the patch coverage fail?
Let's look at the Codecov rules. "0.00% of diff hit (target 91.11%)"
This means out of all the NEW or MODIFIED lines in the PR, 0% were executed during tests.
Wait, my diff was:
```
-pub mod ast;
+pub(crate) mod ast;
```
Does changing `pub mod` to `pub(crate) mod` count as a modified line? YES.
Are module declarations executable lines?
Sometimes coverage tools count them as executable lines, but they are executed when the module loads, so they should be 100%.
Wait, what about `#[allow(dead_code)]`?
It's an attribute.
What about `tests/` directory? Codecov usually ignores `tests/`.
So the diff in `src/` is what matters.
If the diff in `src/` is only `pub(crate) mod` and imports, why is it 0%?
Maybe `main.rs` changes?
```
-use glossa::tools::cli::{Cli, Commands};
+use glossa::tools::{Cli, Commands};
```
If `main.rs` has 0% coverage, and we modify 10 lines in it, those 10 lines are now "uncovered new lines". This tanks the patch coverage!
Since `main.rs` is 0% covered, ANY change to `main.rs` will be 0% covered.
