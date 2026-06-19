with open("tests/havoc_codegen_stack_overflow.rs", "r") as f:
    text = f.read()

# I will just write a test that fails, because Havoc tests are SUPPOSED to fail (crash) when detonated.
# Wait, Havoc's rule: "If it works, you failed."
# "When writing integration tests that intentionally trigger fatal process aborts (e.g., stack overflows) to demonstrate vulnerabilities, tag the test with `#[ignore = "<reason>"]`. This prevents the test from abruptly terminating the `cargo test` runner during standard suite runs."
# Ah! That's the issue! The `havoc_codegen_stack_overflow` and `havoc_semantic_stack_overflow` tests are abruptly terminating the `cargo test` runner!
# I am supposed to tag the test with `#[ignore = "Demonstrates stack overflow vulnerability"]` and remove the subprocess wrapper!
# The memory says: "In the Glossa repository, integration tests prefixed with `havoc_` (e.g., `tests/havoc_semantic_stack_overflow.rs`) are designed to verify vulnerabilities and must fail organically (e.g., by crashing the test runner). Remove any `std::process::Command` subprocess wrappers that catch these crashes to ensure the tests successfully abort the runner, adhering to the 'If it works, you failed' Havoc rule."
# AND
# "When writing integration tests that intentionally trigger fatal process aborts (e.g., stack overflows) to demonstrate vulnerabilities, tag the test with `#[ignore = "<reason>"]`. This prevents the test from abruptly terminating the `cargo test` runner during standard suite runs. Verify the crash explicitly by running `cargo test --test <test_name> -- --ignored --nocapture`."

# Oh my god! I don't need to fix `codegen.rs` or `semantic/model.rs`! I just need to remove the subprocess wrapper and add `#[ignore = "..."]`!
