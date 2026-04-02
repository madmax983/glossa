# 🗣️ Echo: Misleading Troubleshooting Documentation and Silent/ICE Failures

🤦 **The Confusion:**
I was reading the README "Troubleshooting" section, which clearly lists errors like "Οὐκ οἶδα τὸ ὄνομα" (Undefined variable), "Ῥῆμα οὐχ εὑρέθη" (Missing verb), and "Διπλοῦν ὑποκείμενον" (Double Subject). So I tried them to see what the errors look like.
Instead of the nice Greek error messages, the compiler either:
- Silently compiles and runs a sentence with two subjects (`ὁ ἄνθρωπος ὁ θεὸς λέγει.`) and does absolutely nothing.
- Silently compiles and runs an unbound variable (`α λέγε.`) and does absolutely nothing.
- Throws a massive internal compiler panic from `rustc` when a verb is missing (`ὁ ἄνθρωπος τὸν λόγον.` -> `INTERNAL COMPILER ERROR (Codegen Failed)`).

🕵️ **The Reality:**
The documentation promises specific, helpful domain errors, but the actual compiler lacks the validations to produce them. It either generates invalid Rust code that fails during codegen or silently ignores invalid sentences.

💡 **The Fix:**
The semantic analyzer needs to actually check for these conditions (missing verbs, undefined variables, multiple subjects) and emit the precise Greek errors listed in the README, rather than failing late in codegen or succeeding silently.
