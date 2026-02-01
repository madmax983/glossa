## [Reduction]
**Bloat:** `src/semantic/disambiguation.rs` containing purely morphological logic.
**Cut:** Moved to `src/morphology/disambiguation.rs` and re-exported from `crate::morphology`.
**Saved:** Removed "Layer Lasagna" dependency (semantic -> disambiguation -> morphology). Now semantic -> morphology.

## [Fix]
**Issue:** `tests/havoc_stack_overflow.rs` used deprecated `build_ast` API.
**Fix:** Switched to `glossa::parser::parse`.
**Saved:** Fixed a test failure and removed usage of potentially dead API.
