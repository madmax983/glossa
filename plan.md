1. **Target**: Add missing test coverage for `src/semantic/resolver.rs` -> `current_level` method.
2. **Action**: Create and run a python patch script using `run_in_bash_session` to append a new unit test at the end of `src/semantic/resolver.rs`. The test `test_current_level_panic` will forcefully clear the `levels` vector (using `scope.levels.clear()`), and call `define` to trigger the `.expect("Scope must have at least one level")` panic. I will use a single line anchor as requested.
```bash
cat << 'EOF' > patch.py
import os

filepath = "src/semantic/resolver.rs"
with open(filepath, "r") as f:
    content = f.read()

target = "        assert!(scope.lookup(name).is_some());"

addition = """        assert!(scope.lookup(name).is_some());
    }

    #[test]
    #[should_panic(expected = "Scope must have at least one level")]
    fn test_current_level_panic() {
        let mut scope = Scope::new();
        scope.levels.clear(); // Forcefully empty the levels to trigger the panic
        scope.define("test".to_string(), GlossaType::Number); // This calls current_level()"""

new_content = content.replace(target, addition)

with open(filepath, "w") as f:
    f.write(new_content)
EOF
python3 patch.py && rm patch.py
```
3. **Verification**: Run `run_in_bash_session` with `git diff src/semantic/resolver.rs` and `tail -n 30 src/semantic/resolver.rs` to review the applied changes.
4. **Global Test Suite**: Run `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --all` to ensure overall correctness and verify there are no regressions.
5. **Pre-commit**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
6. **Submit**:
Create a PR description with Sentry format using `run_in_bash_session`:
```bash
cat << 'EOF' > PR_DESCRIPTION.md
🛡️ Sentry: [test coverage improvement]

🎯 **Target:** `src/semantic/resolver.rs` -> `Scope::current_level()`
💣 **Risk:** The `current_level()` method uses `.expect("Scope must have at least one level")` on `self.levels`. If the internal vector of scope levels is ever cleared or corrupted programmatically, any definition operation (like `define`, `define_type`, `define_function`) would trigger an unhandled runtime panic. While structural protections exist, this panic branch was entirely uncovered by tests.
🧪 **Strategy:** Wrote a test under `#[cfg(test)] mod tests` in `src/semantic/resolver.rs` that explicitly manipulates the internal state to force an empty `levels` vector (`scope.levels.clear()`), then calls `scope.define()`, verifying that the expected panic message is correctly thrown using `#[should_panic]`.
🔬 **Verification:** Run `cargo test --lib semantic::resolver::tests::test_current_level_panic`
EOF
```
Then use the submit tool.
