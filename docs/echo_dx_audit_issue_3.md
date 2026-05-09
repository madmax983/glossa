# 🗣️ Echo: Philosophy Word Order Example Fails and Wrong Error Docs

🤦 **The Confusion:**
1. I was reading `docs/reference/1-Philosophy.md` about the "Inflection over position" philosophy, and decided to copy-paste the given example to test out the free word order:
```glossa
χρήστης δεδομένα γράφει.    // SOV
γράφει χρήστης δεδομένα.    // VSO
δεδομένα χρήστης γράφει.    // OSV
γράφει δεδομένα χρήστης.    // VOS
```
When I ran it, the compiler literally crashed with a raw Rustc Internal Compiler Error (`Codegen Failed: cannot find value ...`). I thought this was a valid example, but it just crashes the compiler!
2. I also checked the Troubleshooting section in `README.md` which lists the error for undefined variables as `Οὐκ οἶδα τὸ ὄνομα`. However, in practice (when it doesn't just silently evaluate or ICE), the actual compiler error emitted for undefined variables is `Τὸ «...» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό`.

🕵️ **The Reality:**
1. The example code tries to use `χρήστης` and `δεδομένα` without defining them first with `ἔστω`. Because of how `γράφει` compiles to Rust under the hood, passing an undefined variable triggers a fatal codegen abort rather than a friendly error or a silent success.
2. The documentation lies about the exact error string for undefined variables.

💡 **The Fix:**
1. Please add variable declarations to these documentation examples so they are actually runnable!
For example:
```glossa
χρήστης «Plato» ἔστω.
δεδομένα «Book» ἔστω.

χρήστης δεδομένα γράφει.    // SOV
// ...
```
2. Update the `README.md` Troubleshooting section to accurately reflect the exact string `Τὸ «...» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό` instead of `Οὐκ οἶδα τὸ ὄνομα`.
Documentation must be accurate, and examples MUST compile and run correctly out of the box!
