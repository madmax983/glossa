# 022. Encapsulate Internal Modules

Date: 2026-06-12

## Status

Accepted

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This violated the principle of encapsulation by exposing internal implementation details to the public API, creating a sprawling public API and making it difficult to discern which tools were intended for external consumption versus which were strictly internal helpers.

## Decision

We have updated the visibility of these internal modules to restrict them.
Specifically, `src/tools/mod.rs` was modified to change `cache`, `report`, and `ui` to `pub(crate) mod`.
Similarly, `src/semantic/assembly/mod.rs` was modified to change `model` to `pub(crate) mod`.

## Consequences

*   **Positive:** The internal structure is better encapsulated, and internal details don't leak out of their intended domains.
*   **Positive:** The public API surface is smaller and more intentional, achieving higher cohesion.
*   **Negative:** Developers must be conscious of module boundaries when adding new internal utilities.
