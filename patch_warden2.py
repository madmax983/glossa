import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

# Ah! `χ.` is the body of the function!
# `χ.` is a single word expression, representing a return value in a block!
# But `χ.` has NO verb!
# So `check_missing_verb` panics with `MissingVerb`!
# BUT wait! `is_block` is TRUE for `{ χ. }`!
# `check_missing_verb` has:
#         if ctx.is_block { return Ok(()); }
# Why did it NOT return Ok(()) for `{ χ. }` ?
# Let's check `StatementContext`!
