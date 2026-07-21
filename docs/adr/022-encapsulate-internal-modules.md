# 022. Encapsulate Internal Modules

Date: 2026-07-21

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by exposing internal implementation details to the public API and creating a sprawling API surface.

## Decision

We have updated the visibility of these internal modules to `pub(crate) mod` in `src/tools/mod.rs` and `src/semantic/assembly/mod.rs`. This restricts their usage to within the crate while ensuring that they are not accessible to external consumers.

## Consequences

*   **Positive:** The internal structure of the `tools` and `assembly` modules is better encapsulated.
*   **Positive:** The public API of the `glossa` crate is smaller and more intentional.
*   **Negative:** Developers must be conscious of module boundaries and cannot rely on everything being publicly available across the workspace.
