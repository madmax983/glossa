## [Reduction]
**Bloat:** Deep folder hierarchy `src/errors/` with 3 files for < 300 lines of code.
**Cut:** Flattened into single `src/errors.rs`.
**Saved:** Removed directory structure, simplified file navigation.

## [Reduction]
**Bloat:** `src/experimental/bard.rs` implementing a "whimsical English narrative" transpiler. Unused in core compiler.
**Cut:** Deleted the entire module and its CLI command.
**Saved:** ~300 lines of non-essential code. Cognitive load of maintaining a second transpiler.
