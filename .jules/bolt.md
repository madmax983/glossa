# Bolt's Journal ⚡

## [Memory] Zero-Allocation Prefix Matching
**Learning:** Matching strings against a set of prefixes/suffixes is often implemented with `Vec::new()`, `collect()`, and `sort()`. This allocates heap memory on every call. If the patterns are static, pre-sort them in the source code (by length descending for longest-match) and iterate the static slice directly. This turns O(N log N) + Allocation into O(N) stack-only scan.
**Action:** Always check if constant data can be pre-sorted to avoid runtime sorting. Add a regression test to enforce the sort order invariant.

## [Performance] Optimistic vs Pessimistic Checks
**Learning:** Adding a "fast path" check (e.g., iterating to detect if normalization is needed) can cause regressions if the "fast path" fails frequently (dirty data). The double iteration cost outweighs the saved allocation.
**Action:** Profile the data distribution. If data is mostly dirty, optimize the dirty path (e.g. `String::with_capacity`) rather than adding optimistic checks that require re-scanning.

## [Performance] Complex Unicode Casing vs Allocation
**Learning:** `char::to_lowercase` is efficient (iterator) but incorrect for context-sensitive casing (like Greek final sigma). `String::to_lowercase` is correct but allocates. A hybrid approach checking for the presence of uppercase characters allows using the fast path for the common case (lowercase identifiers) while preserving correctness for edge cases.
**Action:** When optimizing casing operations, check if the input is already in a state that allows a simpler, allocation-free transformation.
**[Normalization Caching Strategy]**
**Learning:** When caching a normalized form (e.g. ) in a struct to avoid re-computation, ensure that:
1. The API to populate the struct takes the pre-computed normalized form or computes it once at the boundary.
2. If computing it at the boundary, update all internal helper functions to propagate it, avoiding shadowing or re-computation.
3. Update all test helpers or manual struct instantiations to include the new field.
4. Watch out for  when converting  to  via .
**Action:** When adding a cache field, grep for all instantiations of the struct and all calls to the constructor/feed functions.
