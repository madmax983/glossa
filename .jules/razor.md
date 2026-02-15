## [Reduction]
**Bloat:** The "Bard" experimental module (`src/experimental/bard.rs`) which translated code back to English stories.
**Cut:** Deleted the entire `experimental` module, the `bard` subcommand, and associated tests.
**Saved:** ~300 lines of code / Cognitive load of maintaining a "cute" but non-core feature.

## [Reduction]
**Bloat:** Fragmented `tools` module with tiny files `cache.rs` and `ui.rs`.
**Cut:** Merged `cache.rs` and `ui.rs` into `runner.rs` as private implementation details.
**Saved:** 2 files / Reduced file hopping and import complexity.
