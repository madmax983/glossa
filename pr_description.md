# 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
"Tried to run the troubleshooting examples from `README.md` to trigger 'Οὐκ οἶδα τὸ ὄνομα', 'Διπλοῦν ὑποκείμενον', and 'Ῥῆμα οὐχ εὑρέθη'. Compiler either crashed entirely (INTERNAL COMPILER ERROR) or silently compiled with zero errors, outputting nothing instead of the helpful Greek diagnostic."

🕵️ **The Reality:**
"Turns out the semantic assembler had specific hardcoded values (`ανθρωπος`) ignoring verbs or skipping specific error blocks for variables. Unrecognized variables were silently defaulting to zero (`Unknown`) values without throwing the semantic errors."

💡 **The Fix:**
"Add proper semantic validation logic in `assembler` and `conversion` blocks to correctly throw the missing verb and undefined variable errors instead of compiling silently or crashing the `rustc` pipeline. I've correctly routed unmatched subjects and variables to throw `GlossaError::undefined` displaying the correct Greek error representation."
