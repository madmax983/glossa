1. **Explore and Identify Weak Points:**
   - I have identified `AnalyzedExpr` clone/drop and `generate_rust` stringification as weak points vulnerable to stack overflows.
   - I have also verified that the tests `tests/havoc_clone_drop.rs` and `tests/havoc_codegen_stack_overflow.rs` correctly reproduce these crashes.

2. **Attack - Concurrency Torture (Loom):**
   - Look for `RwLock` or `Mutex` usage. There appear to be none currently in the system, but I should verify `tools/cache.rs` and `tools/runner.rs` to ensure no hidden race conditions exist, especially regarding file operations or environment variables.

3. **Attack - Fuzzing / Proptest:**
   - Write a proptest to target `text::normalize_greek` and `parser::parse` with weird Unicode and boundary conditions.
   - Check if there are public APIs taking `&str` that might panic on garbage data.

4. **Verify Tests:**
   - Ensure the new chaos tests fail in "detonate" mode and pass the test runner by verifying the crash.

5. **Complete pre-commit steps:**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit PR:**
   - Present the "wreckage" in the Havoc format: 👺 Havoc: [TITLE], with 🧨 The Trigger, 📉 The Stack Trace, 🧪 Reproduction, and 😈 Comment.
