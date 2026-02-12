# Razor's Journal

## [Reduction]
**Bloat:** `analyze_participle` allocates and sorts a vector of patterns on every call.
**Cut:** Use `LazyLock` to pre-compute the sorted patterns once.
**Saved:** N/A lines (performance fix), but cleaner logic.

## [Reduction]
**Bloat:** `src/morphology/case.rs` is a small file containing only enums.
**Cut:** Move enums to `src/morphology/mod.rs` and delete `case.rs`.
**Saved:** 1 file, reduced module depth.
