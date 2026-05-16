**Pre-allocating Vec capacity in Morphology Analyzers**
**Learning:** Functions like `analyze_verb_all` and `analyze_noun_all` are called extremely frequently during the semantic analysis phase to test hypotheses about word meanings. Initializing the results vector with `Vec::new()` causes intermediate heap allocations as results are appended.
**Action:** Always inspect loops and hot paths for `Vec::new()` usage where the collection size can be approximated or bounded, and replace with `Vec::with_capacity(n)`.
