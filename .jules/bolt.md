**Avoid Recursive format! Macro Allocations**
**Learning:** When recursively formatting composite structures or trees into strings, using recursive `format!` calls creates unnecessary intermediate heap allocations at each level.
**Action:** Use a helper function that takes a `&mut String` buffer to append directly into a single pre-allocated string instead of using `format!`.
