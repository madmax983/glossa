## FxHashSet Allocation Optimization
**Learning:** Using `FxHashSet<String>` dynamically inside formatting loops causes an `O(N)` heap allocation penalty by calling `.to_string()` on temporary strings. If the strings outlive the data structure scope, `FxHashSet<&str>` avoids allocations entirely.
**Action:** Always verify if a `HashMap` or `HashSet` tracking strings actually needs ownership. If checking against an existing AST or configuration that owns the strings, store `&str` via `.as_str()` to prevent unnecessary heap cloning.
