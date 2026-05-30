1. **Fix Memory Exhaustion DoS in REPL (`src/tools/repl.rs`)**
   - Use `run_in_bash_session` to run `sed` and replace the unbounded `input.read_line(&mut line)` with a capped read `input.by_ref().take(MAX_REPL_SOURCE_LEN as u64).read_line(&mut line)` inside `run_repl_inner` to prevent memory exhaustion on infinite input streams without newlines.
2. **Fix Memory Exhaustion DoS in Mentor (`src/tools/mentor.rs`)**
   - Use `run_in_bash_session` to run `sed` and replace the unbounded `input.read_line(&mut line)` calls (two instances) with a capped read using `.take(50000)` inside `run_mentor_inner` to prevent memory exhaustion.
3. **Verify the fixes**
   - Run the integration test `tests/repl_dos.rs` via `run_in_bash_session` using `cargo test --test repl_dos`.
   - Run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` via `run_in_bash_session` to ensure no regressions or warnings are introduced.
4. **Log the security discovery in Warden's journal**
   - Use `run_in_bash_session` to append a new entry to `.jules/warden.md` detailing the unbounded `read_line` vulnerability and the application of capped readers.
5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
6. **Submit PR**
   - Use `request_code_review` to submit the PR with the required Sentry/Warden format:
     - Title: `🔒 Warden: [Fix Unbounded Read DoS in REPL and Mentor]`
     - Description with:
       * 🦠 Threat: The `read_line` function was used on potentially infinite input streams (without newlines), causing memory exhaustion (DoS).
       * 🛡️ Defense: Applied bounded reading via `.by_ref().take(LIMIT)` to cap maximum memory usage per input line.
       * 💥 Severity: Medium/High (Local Denial of Service).
       * 🧪 Verification: Validated with `tests/repl_dos.rs` simulating an infinite `read` stream without newlines.
