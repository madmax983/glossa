# 022. Encapsulate Internal Modules

Date: 2026-04-22

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by unnecessarily exposing internal implementation details and structures to the public API, leading to a sprawling API surface area.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict the visibility of these modules to `pub(crate) mod`.

## Consequences

- **Positive:** Achieved higher cohesion by keeping the public API surface minimal.
- **Positive:** Ensures internal structures and DTOs do not leak out of their intended domains.
- **Negative:** Internal modules are no longer accessible from outside the crate, requiring careful consideration when integrating new tools.
