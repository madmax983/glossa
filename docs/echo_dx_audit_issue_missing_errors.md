# 🗣️ Echo: Missing Compiler Errors for Troubleshooting Examples

🤦 **The Confusion:**
I read the "Troubleshooting" section in `README.md` and tried to intentionally trigger the documented errors to see what they look like. The documentation says I should get `Διπλοῦν ὑποκείμενον` for a double subject (e.g., `ὁ ἄνθρωπος ὁ θεὸς λέγει.`) and `Οὐκ οἶδα τὸ ὄνομα` for an undefined variable (e.g., `ἄγνωστος λέγε.`). I ran these snippets, but instead of getting a helpful error message, the compiler just silently succeeded and exited without printing anything! Am I doing something wrong, or is the compiler just ignoring my mistakes?

🕵️ **The Reality:**
The compiler actually completely ignores double subjects and undefined variables and silently succeeds, contrary to what the documentation explicitly claims. The user is left staring at a blank terminal instead of a helpful Greek error message.

💡 **The Fix:**
Either the compiler needs to be fixed to actually throw the `Διπλοῦν ὑποκείμενον` and `Οὐκ οἶδα τὸ ὄνομα` errors, or the documentation should stop lying to me about these errors existing. Since I don't fix compiler bugs, please fix this discrepancy so new users aren't confused by silent failures.
