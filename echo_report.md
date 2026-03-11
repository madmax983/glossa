## 🗣️ Echo: Getting Started example is broken

### 🤦 The Confusion
When I follow the "Troubleshooting" guide in the README, I expect the compiler to speak to me in Greek when I make mistakes. The README promises helpful errors like `Οὐκ οἶδα τὸ ὄνομα` (Undefined variable) and `Ῥῆμα οὐχ εὑρέθη` (Missing verb).

However, when I try to run code with these exact issues, the compiler silently fails or executes without error!

### 🕵️ The Reality
1. **Missing Verb:** Running `ξ 10.` (no verb) compiles and executes perfectly with no output and a `0` exit code.
2. **Undefined Variable:** Running `ἄγνωστος λέγε.` just prints an empty line.

The compiler is ignoring these errors completely instead of printing the beautiful Greek error messages promised in the documentation.

### 💡 The Fix
The semantic analyzer needs to actually check for undefined variables and missing verbs, and emit the errors documented in the README. If the compiler cannot detect these, the README is completely misleading.
