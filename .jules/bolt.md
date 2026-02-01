# Bolt's Journal ⚡

## [Memory] Zero-Allocation Prefix Matching
**Learning:** Matching strings against a set of prefixes/suffixes is often implemented with `Vec::new()`, `collect()`, and `sort()`. This allocates heap memory on every call. If the patterns are static, pre-sort them in the source code (by length descending for longest-match) and iterate the static slice directly. This turns O(N log N) + Allocation into O(N) stack-only scan.
**Action:** Always check if constant data can be pre-sorted to avoid runtime sorting. Add a regression test to enforce the sort order invariant.
