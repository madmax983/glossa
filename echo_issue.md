# 🗣️ Echo: 1-Philosophy.md example crashes with Codegen Error

* 🤦 **The Confusion:** Tried to run the first example in `docs/reference/1-Philosophy.md` (the "Inflection over position" code block). The compiler exploded with an `INTERNAL COMPILER ERROR (Codegen Failed)` complaining about undefined variables.
* 🕵️ **The Reality:** The example uses undefined variables (`χρήστης`, `δεδομένα`) which causes the compiler to fail during code generation.
* 💡 **The Fix:** Fix the example so it compiles by either defining the variables first or clearly warning the user that this snippet won't run on its own.
