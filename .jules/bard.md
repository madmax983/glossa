## 2024-05-22 - The Slot-Based Assembler
**Confusion:** Users might assume GLOSSA parses strictly left-to-right like C or Rust.
**Clarification:** The `src/semantic/assembler.rs` module implements a "Slot-Based Assembler" that mimics Ancient Greek grammar. Words are routed to grammatical slots (Subject, Object, Verb, etc.) based on their case endings, not their position. This allows for word-order independence (SOV, VSO, OVS support).
