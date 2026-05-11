# 🗣️ Echo: Free word order examples in Philosophy docs are broken

🤦 **The Confusion:**
I read the documentation in `docs/reference/1-Philosophy.md` about free word order, copied the examples (like `χρήστης δεδομένα γράφει.`), and tried to run them. The compiler exploded with a massive "INTERNAL COMPILER ERROR (Codegen Failed)" and `error[E0425]: cannot find value ... in this scope`.

🕵️ **The Reality:**
Turns out the examples use variables (`χρήστης`, `δεδομένα`) that were never defined anywhere in the code snippet.

💡 **The Fix:**
Please fix the docs! Either add the variable initializations to the code snippets so they actually run. If I copy-paste the example and it doesn't compile, I am leaving.
