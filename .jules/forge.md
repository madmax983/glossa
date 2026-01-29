## 2024-05-22 - Assembler God Object
**Learning:** `src/semantic/assembler.rs`'s `Assembler` struct handles too many distinct parsing states (operators, literals, grammar slots) in a single context. The `feed` method was becoming a dispatch bottleneck.
**Action:** Keep an eye on `Assembler`. Future refactors should consider splitting state (e.g., `OperatorState`, `GrammarState`) or using a state machine pattern instead of a monolithic struct with many `Option` fields.
