# DX Audit Report

🤦 **The Confusion:**
When running the basic `cargo run -- examples/quickstart.γλ` example from the README, the compiler outputs a Rust source code warning: `warning: unused variable: input --> src/main.rs:181:33`.

🕵️ **The Reality:**
In `src/main.rs`, the `input` variable for `Commands::Gnomon` is only used when the `nova` feature is enabled. When `nova` is disabled (the default), `input` becomes unused, leaking a Rust compiler warning to non-Rust users.

💡 **The Fix:**
Add `let _ = input;` inside the `#[cfg(not(feature = "nova"))]` block for `Commands::Gnomon` in `src/main.rs` to silence the warning.
