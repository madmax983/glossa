**[Cartographer `FxHashSet<String>` Optimization]**
**Learning:** `src/tools/cartographer.rs` used `FxHashSet<String>` which required allocating and cloning strings (`to_string()`) when tracking type dependencies. By modifying the internal functions (`format_structs`, `format_dependencies`, `extract_dependencies`) and the map to use `&str` instead of `String`, we could avoid O(N) heap allocations for small internal string operations during cartographer execution.
**Action:** When creating local sets for deduplication, always prefer `FxHashSet<&'a str>` over `FxHashSet<String>` when tracking names tied to ASTs with clear lifespans. This aligns with `idiomatic-rust` and the specific directive to avoid string copies.


**[Cartographer `FxHashSet<String>` Optimization]**
**Learning:** `src/tools/cartographer.rs` used `FxHashSet<String>` which required allocating and cloning strings (`to_string()`) when tracking type dependencies. By modifying the internal functions (`format_structs`, `format_dependencies`, `extract_dependencies`) and the map to use `&str` instead of `String`, we could avoid O(N) heap allocations for small internal string operations during cartographer execution.
**Action:** When creating local sets for deduplication, always prefer `FxHashSet<&'a str>` over `FxHashSet<String>` when tracking names tied to ASTs with clear lifespans. This aligns with `idiomatic-rust` and the specific directive to avoid string copies.
