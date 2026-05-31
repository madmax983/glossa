1. **Analyze Vulnerability**
   - Identify that `src/tools/repl.rs` calls `input.read_line(&mut line)` without limiting the input stream length.
   - An attacker or fuzzing scenario providing an infinite string without a newline `\n` can exhaust memory, resulting in an OOM panic.

2. **Create Havoc Fuzzer Test**
   - Use `run_in_bash_session` with `cat << 'EOF' > tests/havoc_repl_fuzz_oom.rs` to write the fuzzer test.
   - Implement `EndlessGarbage` to generate infinite characters without a newline character, simulating an attacker or network providing infinite garbage.
   - Wrap it in a generous limit via `EndlessGarbage.take(2_000_000_000)` representing memory constraints to ensure completion if bounds aren't checked.
   - Pass this to `run_repl_inner` to prove memory bounds vulnerability.

3. **Verify Vulnerability**
   - Run tests and linters via `run_in_bash_session` to demonstrate the vulnerability without fixing the bug, conforming to Havoc boundaries.

4. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit Wreckage**
   - Use `request_code_review` to create a PR.
   - Title: `👺 Havoc: REPL Memory Exhaustion via Unbounded read_line`
   - Description:
     * 🧨 **The Trigger:** `Input stream of 2 billion bytes without a newline ('\n') caused memory exhaustion in repl::run_repl_inner.`
     * 📉 **The Stack Trace:** `memory allocation of 2000000000 bytes failed` (or similar depending on local memory constraints)
     * 🧪 **Reproduction:** `cargo test --test havoc_repl_fuzz_oom`
     * 😈 **Comment:** `You assumed users would press Enter. You were wrong.`
