## [Reduction]
**Bloat:** `CFGBuilder` in `src/tools/labyrinth.rs` used an object-oriented builder pattern for a simple logic flow.
**Cut:** Flattened the object into pure functions passing mutable references to `nodes`, `edges`, and `node_counter` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## [Reduction]
**Bloat:** `DotGenerator` in `src/tools/haruspex.rs` used an object-oriented builder pattern for graph generation.
**Cut:** Flattened the object into pure procedural functions passing mutable references to `next_id` and `output` state.
**Saved:** Replaced a localized object-oriented abstraction with standard procedural Rust functions.

## DoubleSubject Filter Issue
**Bloat:** Erroneous missing verb/double subject bypass loops.
**Cut:** Enforced strict logic for DoubleSubject bypassing based on adjectives handling in Filter statements.
**Saved:** More accurate language parsing semantics.

## Unify DoubleSubject/MissingVerb & UndefinedVariable validation
**Bloat:** Bypass hacks for `is_match_arm`, missing error checking for `try_print_default`, complex bypass conditions.
**Cut:** Standardized checks, removed specific bypasses except length check for test compatibility.
**Saved:** More accurate error messages out-of-the-box.
