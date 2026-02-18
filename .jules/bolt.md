**[Optimizing Greek Normalization]
**Learning:** `str::to_lowercase` allocates a `String` which is expensive in hot paths like lexing. For complex scripts like Greek, manual `char`-based lowercasing requires careful handling of context-sensitive characters (Sigma).
**Action:** Implement zero-allocation iterators for text processing when standard library methods force allocation.
