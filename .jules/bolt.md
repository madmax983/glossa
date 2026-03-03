# Bolt's Journal

## [Pre-allocate AST node vectors during parsing]
**Learning:** `pest::iterators::Pairs` correctly implements `ExactSizeIterator` (because it is backed by a pre-computed token queue). Therefore, `inner.len()` is O(1) and safe to use.
**Action:** Always use `Vec::with_capacity(inner.len())` when collecting AST nodes from `Pairs` iterators (e.g., in `build_expression`, `build_clauses`) to completely eliminate O(log N) dynamic heap reallocations during the parsing phase.
