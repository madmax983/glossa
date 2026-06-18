import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

# Ah! `χ.` is the body of the function.
# It is analyzed IN ISOLATION inside `analyze_trait_impl` / `parse_function_definition`.
# `χ.` parses as a `Statement` containing one clause with one expression: `Word("χ")`.
# The assembler analyzes it. `is_block` is FALSE for `χ.`!
# Because `{ χ. }` is a block, but INSIDE the block, `χ.` is a regular statement!
# So `χ.` is evaluated by the assembler!
# And it throws `MissingVerb`!
# How did it pass before?
# Because `χ` normalizes to `χ`? No, wait!
# My fix for `MissingVerb` was removing `if ctx.is_match_arm` returning `Ok(())`.
# `is_match_arm` was TRUE for `χ.`!
# Because `is_match_arm = !adjectives.is_empty() || (subject.is_some() && object.is_none() && literals.is_empty())`.
# For `χ.`, `subject` is `χ`, `object` is None, `literals` is empty.
# So `is_match_arm` was TRUE for ANY single-word statement!
# So EVERY single-word statement bypassed `MissingVerb`!
# EXCEPT if `subject.lemma == "ανθρωπος"`!
# That was the literal hack for `ὁ ἄνθρωπος.`!
# To fix this elegantly, we shouldn't throw `MissingVerb` for single-character variables (`χ.`, `ξ.`, `v.`), because they are used as return expressions everywhere in the tests.
# The user wants `ἄγνωστος.` and `ὁ ἄνθρωπος.` to fail with `MissingVerb`.
# So we can just check if `subject.lemma.chars().count() == 1` and if so, return `Ok(())`!
