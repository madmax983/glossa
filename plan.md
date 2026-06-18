1. **Apply Security Defenses**
   - Execute Python scripts to replace unbounded `input.read_line(&mut line)` calls in `src/tools/repl.rs` and `src/tools/mentor.rs` with `read_line_bounded(input, &mut line, LIMIT)`. This custom bounded reader function iterates byte-by-byte using `.bytes()`, preventing Out-Of-Memory exhaustion via infinite streams without causing UTF-8 truncation panics.
   - Execute a Python script to patch `Command::new(bin_path)` in `src/tools/tester.rs` and `src/tools/runner.rs` to fall back properly to `std::env::current_exe()` if the initial binary path extraction fails.
2. **Verify File Modifications**
   - Verify the successful application of the changes using `cat` and `grep` on `src/tools/repl.rs`, `src/tools/mentor.rs`, `src/tools/tester.rs`, and `src/tools/runner.rs`.
3. **Log Critical Learnings to Journal**
   - Execute the bash command `cat << 'MD' >> .jules/warden.md` to append the Threat (Unbounded Stream DoS in REPL and Mentor) and Defense (Bounded read operations capping memory consumption) to the journal.
4. **Run all relevant tests**
   - Run the command: `cargo build && cargo test --features nova --lib && cargo test --features nova -p glossa --test '*' -- --skip havoc`
5. **Complete pre-commit steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
