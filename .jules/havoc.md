## 👺 Havoc: Unsafe Memoization Argument Panic

🧨 **The Trigger:** Passing arguments into an anonymous function with `CaptureMode::Memoize` (e.g. perfect participle with iterators).

📉 **The Stack Trace:**
```
thread 'test_memoize_arguments_panic' panicked at src/codegen.rs:1101:9:
Memoization is only supported for 0-argument closures
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: glossa::codegen::generate_memoized_closure
   3: glossa::codegen::generate_closure
   ...
```

🧪 **Reproduction:** Run `cargo test test_memoize_arguments_panic` from `tests/havoc_proptest_memoize.rs`.

😈 **Comment:** You assumed memoized closures would never be fed arguments because they are supposed to be "thunks". You were wrong. If we pass arguments to a perfect participle mapping, it triggers the panic you left behind as a "security check" because you were too lazy to implement a real argument-based cache key. If it crashes, I win.
