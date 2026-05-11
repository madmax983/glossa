**[Cartographer String Allocation Reduction]**
**Learning:** `FxHashSet<String>` tracking dependencies during struct formatting requires mapping short struct names from `SmolStr` into heap-allocated `String`s via `.to_string()`. Because the definitions already live in `program.scope` inside `AnalyzedProgram`, we can reference them directly using `&str`.
**Action:** When tracking short-lived struct names or references from a larger AST/Program struct, pass `&'a str` to `FxHashSet` instead of `String` and `.as_str()` out of `SmolStr` to avoid unnecessary heap allocations on the hot path.
