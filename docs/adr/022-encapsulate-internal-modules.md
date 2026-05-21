# 022. Encapsulate Internal Modules

Date: 2026-05-21

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by exposing internal implementation details to the public API, creating a sprawling surface area and allowing potential leaks of internal structures out of their intended domains.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these modules with `pub(crate) mod`. This ensures they are only accessible within the crate and are not part of the public API.

## Consequences

*   **Positive:** Achieved higher cohesion and better separation of concerns.
*   **Positive:** The public API surface is minimized, preventing internal structures from leaking out of their domains.
*   **Negative:** Developers must use crate-internal paths and cannot rely on these modules being part of the public API if used externally (which they shouldn't be).
