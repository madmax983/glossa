# 22. Split Lexicon and Conversion

Date: 2026-05-11
Status: Proposed

## Context

The files `src/morphology/lexicon.rs` and `src/semantic/conversion.rs` had grown significantly large, exhibiting the "Blob" anti-pattern. A significant portion of this size was due to extensive inline tests. This massive file size was hurting cohesion, navigating the codebase, and developer experience.

## Decision

We converted both `src/morphology/lexicon.rs` and `src/semantic/conversion.rs` into directory modules (`src/morphology/lexicon/mod.rs` and `src/semantic/conversion/mod.rs`) and extracted their respective tests into new `tests.rs` sub-modules.

## Consequences

- File size for the core logic modules is significantly reduced.
- Code navigation and readability are improved by cleanly separating test code from core logic.
- We have introduced new module boundary declarations and imports, but this trade-off is worthwhile for the organizational benefits.
