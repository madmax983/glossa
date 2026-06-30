## 🗣️ Echo: Missing Error Messages in Troubleshooting Guide

🤦 **The Confusion:** The troubleshooting guide explicitly claims the compiler gives friendly Greek error messages for missing verbs, undefined variables, and double subjects. But when I copy-paste the examples, it silently parses them or completely crashes rustc! The messages were missing.
🕵️ **The Reality:** I traced it down to `src/semantic/assembly/mod.rs` overriding rules directly for `is_print_verb` and an explicit string "ανθρωπος", ignoring the errors. Furthermore, undefined variables in `conversion.rs` were silently turning into zero (`GlossaType::Unknown` fallback).
💡 **The Fix:** Removed the buggy exceptions in the assembler and added undefined variable assertions within the expression extraction loops, fixing the tests locally.
