To fix the patch coverage issue, we should revert changes in `src/main.rs`!
How? By restoring the original module paths that `main.rs` uses, so we don't have to touch it.
BUT, `Atlas` persona rules state: "Upgraded `pub mod` to `pub(crate) mod` across internal modules".
If `src/tools/mod.rs` uses `pub(crate) mod cli;`, then `glossa::tools::cli::Cli` is private.
If we add `pub mod cli { pub use crate::tools::cli_impl::*; }`, we'd have to rename the file to `cli_impl.rs`, which is a messy workaround.

What if we just add a test for `main.rs` so that it's covered?
Let's see `src/main.rs`. It's just a CLI wrapper.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parse() {
        // Just parsing some args to cover the `Cli::parse()` part if we could, but main() does `Cli::parse()`.
        // We can't call main() without it reading real args.
    }
}
```

Actually, `tarpaulin` allows you to ignore files!
Does the project already ignore `main.rs`?
No, the coverage report says:
`main.rs 58 58 0.00% 1 1 0.00% 30 30 0.00% 0 0 -`

If `main.rs` is 0%, ANY touch to it drops patch coverage.
We should add `#[cfg(not(tarpaulin_include))]` to the functions we modified, OR just to `main()`?
If we add `#[cfg(not(tarpaulin_include))]` to `main()`, we modify lines in `main.rs` which will now be ignored by coverage, so they won't count against patch coverage.

Wait! The easiest way to deal with this is to NOT modify `src/main.rs`.
If we don't modify `src/main.rs`, how do we fix the import errors?
Currently `main.rs` imports:
`use glossa::tools::cli::{Cli, Commands};`
`use glossa::tools::dictionary::lookup_word;`
`use glossa::tools::repl::run_repl;`
`use glossa::tools::runner::{bard_file, build_file, check_file, highlight_file, run_file};`

If we want `glossa::tools::cli::{Cli, Commands}` to be accessible, `cli` MUST be a public module.
BUT `Atlas` says "Internal submodules across tools ... were publicly exposed using pub mod. Blueprint: Upgraded pub mod to pub(crate) mod".
If `cli` is an "internal submodule", then the binary `main.rs` shouldn't know about it. `main.rs` should just do `use glossa::tools::{Cli, Commands};` because `cli` is an implementation detail.
The problem is that changing `main.rs` triggers the 0% coverage rule for those changed lines.

Wait, if I add `#[cfg(not(tarpaulin_include))]` to the top of `src/main.rs`, then ALL of `main.rs` is ignored for coverage. This is actually a standard practice for binary wrappers that just call library code.
Wait, does Codecov also use `llvm-cov`? Yes. Does `#[cfg(not(tarpaulin_include))]` work for `llvm-cov`? No, `#[coverage(off)]` is unstable. For `llvm-cov`, you usually use `// coverage:ignore-file` or `#[cfg(not(coverage))]` (unstable).

Let's check if there's any file in the repo that ignores coverage.
