🌟 Nova: The Simulator (ὁ Ὑποκριτής)

💡 **The Spark:** I noticed we have a fully functional tree-walk interpreter (`src/tools/interpreter.rs`) that was isolated and only used for internal tests. Why compile to Rust for simple scripts when we can just simulate them immediately?

🚀 **The Feature:** Implemented `glossa simulate <file>`, which parses, analyzes, and executes a program directly in-memory using the tree-walk interpreter.

🔮 **The Potential:** This enables a lightning-fast execution mode, skipping `rustc` entirely. It paves the way for interactive web playgrounds (WASM) where invoking `rustc` is impossible.

⚠️ **Risk:** Low. Completely isolated behind the `nova` feature flag as an optional CLI command.

📝 **Assumptions:** I observed intentional panics (stack overflows) in the existing `havoc_*` tests during verification, which are assumed to be expected behavior from the Chaos Engineer (`Havoc`) persona and were not patched.
