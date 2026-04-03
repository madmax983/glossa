# 🗣️ Echo: Misleading Troubleshooting Documentation and Unhelpful Errors

🤦 **The Confusion:** The `README.md` Troubleshooting section claims that missing variables trigger an `Οὐκ οἶδα τὸ ὄνομα` (I don't know the name) error, and missing verbs trigger a `Ῥῆμα οὐχ εὑρέθη` (Verb not found) error. However, when I try to run code with a missing verb (`ὄνομα.`), it produces a generic `Σφάλμα συντάξεως: Parse error`. Worse, when I run a script with just an undefined variable (`α.`), the compiler fails with a `Σφάλμα κώδικος (Codegen Error)`.

🕵️ **The Reality:** The Glossa compiler does not actually implement the friendly, translated error messages promised in the README. Instead, syntax errors fall back to raw parser failures, and semantic errors (like undefined variables) bypass validation and cause codegen failures. Furthermore, CLI errors (like file not found) are printed entirely in Greek (`Ἀρχεῖον οὐχ εὑρέθη`), which is intimidating for new users trying to debug basic CLI issues.

💡 **The Fix:** Either implement the semantic validation and friendly error messages promised in the Troubleshooting table (e.g., catching undefined variables before codegen), or update the README to reflect the current, less-friendly reality. Also, translate basic CLI errors (like 'file not found') to English so users know what went wrong before they learn Greek.
