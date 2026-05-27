1.  **Write a test to prove memory exhaustion in REPL**
    - The memory states: "To prevent memory exhaustion (DoS) vulnerabilities when reading potentially unbounded input streams (like `/dev/zero` or standard input), wrap the reader using `std::io::Read::take(LIMIT)` before reading. For `BufRead` types like `StdinLock`, use `input.by_ref().take(LIMIT)` to safely cap methods like `read_line` or `read_to_string`."
    - We will write a test in `tests/havoc_repl.rs` that verifies `run_repl_inner` memory exhaustion bound on unbounded streams.
    - We will use `take` limit of `MAX_REPL_SOURCE_LEN * 2` on the test stream (to guarantee `cargo test` terminates), while testing a bug that doesn't limit `input`.
2.  **Run the test to verify it fails due to stack overflow/memory exhaustion (SIGKILL)**
    - This creates the wreckage to satisfy Havoc.
3.  **Submit PR**
    - Create PR with title `👺 Havoc: [TITLE]` and specific description.
    - Title: "👺 Havoc: Memory exhaustion in REPL"
    - Description:
        * 🧨 **The Trigger:** "Piping an infinite stream of spaces (e.g. `yes " "`) into the REPL causes unbounded reads, allocating memory until the process is OOM-killed."
        * 📉 **The Stack Trace:** (OOM Kill / timeout from test output)
        * 🧪 **Reproduction:** "Run `cargo test --test havoc_repl`."
        * 😈 **Comment:** "You assumed standard input would always come from a human typing slowly. You were wrong."
    - Note: Havoc persona strictly forbids fixing the bug!
