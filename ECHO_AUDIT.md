# 🗣️ Echo: Troubleshooting guide error messages are broken and missing

🤦 **The Confusion:**
I was following the "Troubleshooting" guide in the `README.md` to see how the Ancient Greek error messages work. It advertises friendly messages like `Οὐκ οἶδα τὸ ὄνομα` (Undefined variable) and `Διπλοῦν ὑποκείμενον` (Double Subject). So I tried to trigger them:
1. I tried to trigger the undefined variable by writing `ἄγνωστος λέγε.`.
2. I tried to trigger the double subject error by writing `ὁ ἄνθρωπος ὁ θεὸς λέγει.`.

🕵️ **The Reality:**
Neither of these friendly error messages appeared! Instead, both of these invalid code snippets compiled cleanly and completely silently! The promised errors in the documentation simply do not exist or are completely ignored by the compiler.
Additionally, when I accidentally used an `i64` variable as a map by calling `τίθησι` on it, the compiler bypassed semantic checks and threw a terrifying, massive raw `rustc` abort error screaming about an `INTERNAL COMPILER ERROR (Codegen Failed)` instead of a helpful Glossa type error.

💡 **The Fix:**
Please fix the semantic validation phase! The compiler needs to accurately catch undefined variables and double subjects so that it can emit the beautiful Greek error messages promised in the README. It also needs to validate type operations before code generation to prevent internal Rust errors from leaking to users.
