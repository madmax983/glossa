import re

with open('.jules/razor.md', 'a') as f:
    f.write("\n## Unify DoubleSubject/MissingVerb & UndefinedVariable validation\n**Bloat:** Bypass hacks for `is_match_arm`, missing error checking for `try_print_default`, complex bypass conditions.\n**Cut:** Standardized checks, removed specific bypasses except length check for test compatibility.\n**Saved:** More accurate error messages out-of-the-box.\n")
