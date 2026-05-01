1. Modify `src/tools/scholar.rs` to replace `fs::read_to_string` with bounded reads using `std::io::Read::take()` to prevent DoS vulnerability during testing. Although the vulnerability occurs in test code, we will enforce strict limits.
2. The unbounded read is in `test_run_scholar_empty_fields_methods_functions` and `test_run_scholar_with_functions` in `src/tools/scholar.rs`.
3. Update `.jules/warden.md` with the new vulnerability finding and defense.
4. Execute `cargo clippy`, `cargo test`, and `cargo fmt`.
5. Pre-commit check.
6. Submit PR.
