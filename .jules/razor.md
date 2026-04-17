## [Reduction]
**Bloat:** [Brittle index-based string slicing (`rfind(" ... ")`) in `src/tools/tester.rs`]
**Cut:** [Replaced with sequential iterator consumption `split_whitespace()` for clean token parsing]
**Saved:** [14 lines of string parsing boilerplate and unchecked indexing logic]
