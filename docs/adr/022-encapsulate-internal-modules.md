# 022. Encapsulate Internal Modules

Date: 2026-07-25

## Status

Accepted

## Context

As the ΓΛΩΣΣΑ compiler grew, several internal helper modules under the `src/tools/` directory (specifically `cache`, `report`, and `ui`) and the `src/semantic/assembly/` directory (`model`) were exposed as `pub mod`. This broke encapsulation by unintentionally exposing internal implementation details and DTOs to the public API of the compiler crate. A sprawling public API makes it difficult for consumers to discern which tools are intended for external use and which are strictly internal helpers.

## Decision

We restricted the visibility of these internal modules to `pub(crate) mod`.
Specifically, `src/tools/mod.rs` was updated to encapsulate `cache`, `report`, and `ui`.
`src/semantic/assembly/mod.rs` was updated to encapsulate `model`.
Public structures that are still needed externally (like `Cache`) are explicitly re-exported using `pub use`.

## Consequences

*   **Positive:** The public API surface is now minimal and intentional, achieving higher cohesion.
*   **Positive:** Internal structures and DTOs no longer leak out of their intended domains, preventing external dependencies on implementation details.
*   **Negative:** Internal modules can no longer be directly accessed by external integration tests, requiring tests to go through public facades.
