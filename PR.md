🛡️ Sentry: [test coverage improvement]

🎯 Target: `src/codegen.rs`, `src/tools/auditor.rs`
💣 Risk:
- `codegen.rs`: Missed logic for operator fallbacks (`<=`, `>=`, string concatenation `+`, etc.) could drift or produce invalid tokens silently if unexercised. Unary operator `unwrap()` and overflow panics for numbers (`checked_neg().expect(...)`) were similarly absent in tests, which could hide compiler bugs.
- `auditor.rs`: Missing unwrap coverage when generating files in tests could hide missing capabilities or error paths in the IO manipulation logic inside `run_auditor`.

🧪 Strategy:
- Added `test_generate_unreachable_operators_all_unreachable_branches_all_ops` to systematically iterate all non-numeric binary operators (falling through to the catch-all matcher in `generate_bin_op`).
- Added `test_codegen_unary_neg_overflow_expect` to ensure `UnaryOp::Neg` properly tests numeric and non-numeric representations (`checked_neg` wrapper vs simple `-`).
- Added `test_auditor_output_success_and_issues_unwrap` in the auditor module to correctly ensure temporary path unwraps don't fail, tracking different diagnostic paths directly using the tempdir standard testing pattern.
- Resolved private module visibility (`E0603`) issues that prevented previous attempts in `tests/` by placing tests within internal `#[cfg(test)] mod tests`.

🔬 Verification:
Run the tests locally utilizing standard Rust functionality:
```bash
cargo test test_codegen_unary_neg_overflow_expect
cargo test test_generate_unreachable_operators_all_unreachable_branches_all_ops
cargo test test_auditor_output_success_and_issues_unwrap
```
