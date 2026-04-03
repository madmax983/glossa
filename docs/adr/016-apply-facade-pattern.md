# 16. Apply Facade Pattern to Enforce API Boundaries

Date: 2026-04-01
Status: Proposed

## Context

Atlas refactored several internal public modules (`pub mod`) in `morphology`, `parser`, `semantic`, and `tools` to crate-visibility (`pub(crate) mod`). This prevents downstream clients from depending on unstable internal sub-modules (leaky abstractions).

## Decision

We use the Facade pattern by restricting internal module visibility to `pub(crate)` and explicitly re-exporting necessary items using `pub use` at the crate or top-level module boundaries.

## Consequences

This strictly enforces public API boundaries and hides implementation details. However, it requires maintaining explicit `pub use` statements and updating any existing integration tests to point to the new flattened public API.
