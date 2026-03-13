# 👺 Havoc: Array Out-of-Bounds / Negative Index Panic

🧨 **The Trigger:**
Providing a negative integer as an index to an array collection (e.g., `[1, 2, 3]`) triggers a direct runtime panic in the generated Rust code.
Example code: `ξ [1, 2, 3] ἔστω. ψ -1 ἔστω. ξ ψ μέρος λέγε.`

📉 **The Stack Trace:**
```
thread 'main' panicked at 'Negative index access: -1', src/main.rs:10:20
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

🧪 **Reproduction:**
```rust
use glossa::parser::parse;
use glossa::semantic::analyzer::analyze_program;
use glossa::tools::interpreter::Interpreter;

#[test]
fn test_negative_index_crash() {
    let code = "ξ [1, 2, 3] ἔστω. ξ -1 μέρος λέγε.";
    let ast = parse(code).unwrap();
    let program = analyze_program(&ast).unwrap();
    let mut interpreter = Interpreter::new();
    let _ = interpreter.run(&program); // -> Panics with Parse Error or Runtime Error depending on eval mode.
}
```
*Note: In generated Rust code (`generate_rust`), negative indices generate an explicit `panic!("Negative index access: {}", idx);` branch that deterministically crashes the compiled executable.*

😈 **Comment:**
"You assumed users would only use positive indexes. The `try_from` safely handles platforms that can't fit huge indexes, but negative values cause an intentional panic instead of gracefully failing with `Option` or `Result`. If I can crash your generated executable, I win."
