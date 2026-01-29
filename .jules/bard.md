# Bard's Journal 🎻

## 2024-05-22 - The Assembler's Burden

**Confusion:** The `Assembler` struct in `src/semantic/assembler.rs` is a "God Object" that manages parsing state for every possible sentence type (simple statements, bindings, control flow, loops, etc.). It has a huge number of `pending_*` fields (16 at last count), making it difficult to reason about which fields are valid in which context.

**Clarification:** I added extensive documentation to `src/semantic/assembler.rs` to explain the "Slot-Based Assembly" concept. The key insight is that the `Assembler` acts as a bucket for grammatical cases.
- **Nominative** -> Subject slot
- **Accusative** -> Object slot
- **Dative** -> Indirect object slot

However, this design means the `Assembler` must handle *all* possible combinations, leading to its complexity. Future refactoring should consider splitting the `Assembler` into smaller, specialized assemblers (e.g., `PredicateAssembler`, `LoopAssembler`) or using a more formal state machine transition system.

For now, the documentation clarifies *how* it works, even if the implementation is heavy.
