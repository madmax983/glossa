**[AST Depth Verification]
**Learning:** Raw AST limits enforce bounded memory and prevent stack overflows on pathological deeply-nested input. However, earlier test coverage missed the error paths triggering these limits. A false sense of safety can occur if the error paths aren't explicit tested to return LimitExceeded or SemanticError.
**Action:** Always assert the exact error enum variants and payload messages inside the Err case when adding limit validation tests.
