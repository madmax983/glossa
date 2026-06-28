🛡️ Sentry: [test coverage improvement]

🎯 **Target:** `src/semantic/resolver.rs` -> `Scope::current_level()`
💣 **Risk:** The `current_level()` method uses `.expect("Scope must have at least one level")` on `self.levels`. If the internal vector of scope levels is ever cleared or corrupted programmatically, any definition operation (like `define`, `define_type`, `define_function`) would trigger an unhandled runtime panic. While structural protections exist, this panic branch was entirely uncovered by tests.
🧪 **Strategy:** Wrote a test under `#[cfg(test)] mod tests` in `src/semantic/resolver.rs` that explicitly manipulates the internal state to force an empty `levels` vector (`scope.levels.clear()`), then calls `scope.define()`, verifying that the expected panic message is correctly thrown using `#[should_panic]`.
🔬 **Verification:** Run `cargo test --lib semantic::resolver::tests::test_current_level_panic`
