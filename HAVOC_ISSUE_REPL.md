👺 Havoc: REPL Denial of Service (Empty Statement List)

🧨 **The Trigger:**
Providing an empty string, pure whitespace (`" "`), or pure comments to the REPL causes the parser to return an empty statement list. The REPL context blindly `.unwrap()`s the `.last()` element of this array, causing a Denial of Service.

📉 **The Stack Trace:**
```
thread 'main' panicked at src/tools/repl.rs:350:52:
called `Option::unwrap()` on a `None` value
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: core::panicking::panic
   3: core::option::unwrap_failed
   4: glossa::tools::repl::ReplContext::execute
   5: glossa::tools::repl::run_repl_inner
```

🧪 **Reproduction:**
Run the REPL using `cargo run -- repl` and hit enter (sending an empty string/whitespace) or type `«comment»`. The thread will panic immediately. Alternatively, `cargo test --test havoc_repl_empty` proves the fragility of `.last().unwrap()` directly.

😈 **Comment:**
You assumed the user would always type valid code. Sometimes, silence is deadly.
