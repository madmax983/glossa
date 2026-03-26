**[Optimized join over iteration in narrator.rs]**
**Learning:** Found several places where `exprs.iter().map(tell_expr).collect::<Vec<String>>().join(", ")` was allocating an intermediate `Vec<String>`.
**Action:** Replaced with a helper `join_tell_exprs` which iterates, concatenates directly into a `String` with capacity hints where needed.

**[Optimized join over iteration in narrator.rs]**
**Learning:** Found several places where `exprs.iter().map(tell_expr).collect::<Vec<String>>().join(", ")` was allocating an intermediate `Vec<String>`.
**Action:** Replaced with a helper `join_tell_exprs` which iterates, concatenates directly into a `String` with capacity hints where needed.
