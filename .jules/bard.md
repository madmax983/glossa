## 2026-01-28 - The Slot-Based Assembler
**Confusion:** Users might assume GLOSSA parses strictly left-to-right like C or Rust.
**Clarification:** The `src/semantic/assembler.rs` module implements a "Slot-Based Assembler" that mimics Ancient Greek grammar. Words are routed to grammatical slots (Subject, Object, Verb, etc.) based on their case endings, not their position. This allows for word-order independence (SOV, VSO, OVS support).
# Bard's Journal 🎻

## 2026-01-28 - The Assembler's Burden

**Confusion:** The `Assembler` struct in `src/semantic/assembler.rs` is a "God Object" that manages parsing state for every possible sentence type (simple statements, bindings, control flow, loops, etc.). It has a huge number of `pending_*` fields (16 at last count), making it difficult to reason about which fields are valid in which context.

**Clarification:** I added extensive documentation to `src/semantic/assembler.rs` to explain the "Slot-Based Assembly" concept. The key insight is that the `Assembler` acts as a bucket for grammatical cases.
- **Nominative** -> Subject slot
- **Accusative** -> Object slot
- **Dative** -> Indirect object slot

However, this design means the `Assembler` must handle *all* possible combinations, leading to its complexity. Future refactoring should consider splitting the `Assembler` into smaller, specialized assemblers (e.g., `PredicateAssembler`, `LoopAssembler`) or using a more formal state machine transition system.

For now, the documentation clarifies *how* it works, even if the implementation is heavy.

## 2026-01-29 - The Conversational REPL

**Confusion:** Users (and devs) might expect the REPL to work like Python's, where statements are executed and state is mutated in memory.

**Clarification:** Documented that the ΓΛΩΣΣΑ REPL uses a "Conversational" model where the *entire history* is re-compiled on every line. This ensures consistent scope resolution without duplicating compiler logic, but relies on the compiler being fast enough. I added limits (`MAX_REPL_BINDINGS`) to prevent this from spiraling out of control.
