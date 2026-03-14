# 🗣️ Echo: Internal Compiler Error leaks raw Rustc details on type mismatch

🤦 **The Confusion:**
"I wrote a simple program where I tried to use a number as a map (`ξ 10 ἔστω. «χαῖρε» ξ τίθησι.`). Instead of a helpful error telling me 'ξ is not a map', the compiler crashed with an enormous red box saying `INTERNAL COMPILER ERROR (Codegen Failed)` and spit out a raw Rust error (`error[E0599]: no method named insert found for type i64`)."

🕵️ **The Reality:**
"The compiler transpiles to Rust, but doesn't check types thoroughly enough before generating the code. So when I write semantically invalid code, it generates invalid Rust code, and then `rustc` fails and yells at me instead of Glossa catching the error."

💡 **The Fix:**
"Add semantic type checking for operations like map insertion before code generation, and return a clean, helpful Glossa error (e.g., 'Type mismatch') instead of leaking raw `rustc` stack traces."
