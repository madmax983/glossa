# 🗣️ Echo: Getting Started and Error Checking are broken

🤦 **The Confusion:**
1. I followed the README's literal instructions: `cargo run --release -- test my_tests.γλ`. It immediately threw `Ἀρχεῖον οὐχ εὑρέθη` (File not found) because the file `my_tests.γλ` doesn't exist.
2. The README promised me helpful Greek error messages like "Οὐκ οἶδα τὸ ὄνομα" (Undefined variable). But when I purposefully typed `ἄγνωστος λέγε.` (a variable I never defined), the program compiled and ran with no output! It just swallowed my bug!
3. Even worse, when I tried to trigger the "Missing verb" error (`Ῥῆμα οὐχ εὑρέθη`) by just typing `ὁ ἄνθρωπος.`, the compiler exploded with a massive Rust `rustc` Internal Compiler Error (`error[E0425]: cannot find value ... in this scope`) instead of the helpful Greek message. I am terrified of this stack trace.

🕵️ **The Reality:**
1. The README assumes I know that `my_tests.γλ` is just a placeholder name and that I need to create it myself.
2. The error handling for undefined variables doesn't seem to work—it silently ignores them instead of printing the promised Greek error.
3. The error handling for missing verbs is broken, causing the generated Rust code to fail compilation and dump the raw Rust compiler error on the user.

💡 **The Fix:**
1. Change the test run command in the README to point to a file that actually exists, like `cargo run --release -- test examples/working_tests.γλ`.
2. Fix the variable scoping check in the compiler so it actually throws `Οὐκ οἶδα τὸ ὄνομα`.
3. Catch missing verbs during parsing/semantic analysis so they return `Ῥῆμα οὐχ εὑρέθη` instead of triggering an internal Rust compiler panic.
