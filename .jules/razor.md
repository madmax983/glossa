## [Reduction]
**Bloat:** `GnomonVisitor` and `AuditorVisitor` structs were functioning as unnecessary boilerplate state-holders for tree traversal (object-oriented abstractions).
**Cut:** Removed the visitor structs and replaced them with direct, explicit procedural recursive functions (`visit_statement_gnomon`, `visit_statement_auditor`, etc.). Single-use helper methods were also inlined.
**Saved:** Reduced cognitive load and unnecessary abstractions by utilizing pure functions passing mutable references to explicitly needed state.
