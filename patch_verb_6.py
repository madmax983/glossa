# The memory rule says: "enforce 'Missing Verb' and 'Double Subject' checks during statement classification (`classify_expression` in `src/semantic/conversion.rs`), ensuring you ignore valid verbless forms like queries, blocks, or binary operator fallbacks."
# A verbless form inside a block... how do we know it's inside a block?
# `analyze_statement` calls `extract_block_statements` to parse statements inside a `{ }`.
# Inside the block, the statement is just `χ.`
# But we don't know it's inside a block from `asm_stmt` because we are analyzing it individually.
# Wait! `χ.` evaluates to `Subject("χ")`. Is returning it directly the only issue?
# A block expression in Rust ` { x } ` returns `x`. It's a statement with no semicolon.
# In Glossa, `χ.` means return `x`.
# Why did `test_missing_verb.gl` `ὁ ἄνθρωπος.` fail if `χ.` is valid?
# Ah! `ὁ ἄνθρωπος.` failed codegen because `ἄνθρωπος` is NOT A DEFINED VARIABLE.
# Wait, `ὁ ἄνθρωπος.` compiled without semantic error to `g__u3b1__u3bd__u3b8__u3c1__u3c9__u3c0__u3bf__u3c2_ ;`, which then failed rustc because it's undefined!
# YES! The issue was `ἄγνωστος λέγε.` compiled to 0 silently.
# `ὁ ἄνθρωπος.` compiled to an undefined variable, which rustc caught.
# ECHO ISSUE:
# "The missing verb `ὁ ἄνθρωπος.` just straight up threw an **INTERNAL COMPILER ERROR (Codegen Failed)** with raw rustc output! The only one that works is Ἀσυμφωνία (Disagreement)."
# Oh, so my `MissingVerb` check is correct. The problem is `χ.` should ALSO be an error UNLESS it is the last expression of a block!
# Or maybe `χ.` is valid, but the user expects `ὁ ἄνθρωπος.` to return `MissingVerb`.
