# 🗣️ Echo: Error messages in Troubleshooting guide don't exist

🤦 **The Confusion:**
I was reading the `README.md` and saw the "Troubleshooting" section with some cool Ancient Greek error messages like "Οὐκ οἶδα τὸ ὄνομα" (Undefined variable), "Διπλοῦν ὑποκείμενον" (Double Subject), and "Ῥῆμα οὐχ εὑρέθη" (Missing verb). So, I tried to trigger them to learn how they work. I literally copy-pasted bad code like `ἄγνωστος λέγε.` (undefined variable) and `ὁ ἄνθρωπος.` (missing verb).

🕵️ **The Reality:**
None of these errors actually show up!
- The undefined variable just compiled cleanly without telling me anything (apparently it evaluates to `0` in the background and silently fails because `src/semantic/expressions.rs` returns a generic `GlossaError::semantic` instead of `GlossaError::undefined` AND `extract_value` in `src/semantic/conversion.rs` defaults unextractable variables to `0`).
- The double subject `ὁ ἄνθρωπος ὁ θεὸς λέγει.` also compiled with zero errors (the `Assembler` stashes extra subjects in `self.state.nominatives` for function calls, but `classify_expression` in `src/semantic/conversion.rs` never throws `AssemblyError::DoubleSubject` when those nominatives go unused).
- The missing verb `ὁ ἄνθρωπος.` just straight up threw an **INTERNAL COMPILER ERROR (Codegen Failed)** with raw rustc output! (`classify_expression` blindly pushes undefined subject variables into the AST without a verb).

💡 **The Fix:**
Please either implement these helpful error messages so the compiler actually uses them, or remove them from the README! It is extremely confusing to tell users they will see friendly Greek errors when instead they get silent compilation or raw `rustc` aborts. Also, an undefined variable shouldn't just silently become 0!