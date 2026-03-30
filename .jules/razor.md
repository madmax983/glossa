## Remove Empty Impl Blocks and Optimize Joins
**Bloat:** Empty `impl BinaryOp {}` and `impl UnaryOp {}` in `src/morphology/lexicon.rs`, and an unnecessary `.collect::<Vec<_>>().join(", ")` pattern allocating heap space in `src/tools/report.rs`.
**Cut:** Deleted the empty impl blocks completely. Replaced the `collect` pattern with an iterative `write!` loop on a pre-allocated string.
**Saved:** 4 lines of dead code and intermediate O(N) heap allocations for function parameter reporting.
