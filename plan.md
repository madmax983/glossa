1. **Fix `clippy::useless_borrows_in_formatting` in `src/semantic/conversion.rs`**
   - The CI failed with:
     ```
     error: redundant reference in `format!` argument
     --> src/semantic/conversion.rs:512:13
      |
     512 |             &var_name
      |             ^^^^^^^^^ help: remove the redundant `&`: `var_name`
     ```
   - I will use `replace_with_git_merge_diff` to remove the redundant `&` before `var_name` on line 512 in `src/semantic/conversion.rs`.
   - The exact replacement will be:
```rust
<<<<<<< SEARCH
        Some(b) if !b.mutable => Err(GlossaError::semantic(format!(
            "Τὸ «{}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ",
            &var_name
        ))),
=======
        Some(b) if !b.mutable => Err(GlossaError::semantic(format!(
            "Τὸ «{}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ",
            var_name
        ))),
>>>>>>> REPLACE
```

2. **Verify changes in `src/semantic/conversion.rs`**
   - Use `run_in_bash_session` to run `cat src/semantic/conversion.rs | sed -n '505,515p'` to visually inspect the fix.

3. **Verify tests and clippy pass**
   - Use `run_in_bash_session` to run `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test` to ensure all checks pass.

4. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit PR**
   - Use the `submit` tool to update the PR with the new fix.
