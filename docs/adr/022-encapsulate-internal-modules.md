# 022. Encapsulate Internal Modules

Date: 2026-06-25

## Status

Accepted

## Context

Certain modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This violated encapsulation principles by unnecessarily exposing internal implementation details to the public API.

## Decision

We updated the visibility of these internal modules to `pub(crate) mod` in their respective `mod.rs` files (`src/tools/mod.rs` and `src/semantic/assembly/mod.rs`).

## Consequences

*   **Positive:** The internal structure of the `tools` and `assembly` modules is better encapsulated.
*   **Positive:** The public API surface is minimized, leading to higher cohesion.
*   **Negative:** Developers must respect module boundaries when adding internal utilities.
