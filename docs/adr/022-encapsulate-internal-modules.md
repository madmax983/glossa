# 022. Encapsulate Internal Modules

Date: 2026-08-07

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by exposing internal implementation details to the public API, leading to a sprawling API surface and potential misuse of internal structures.

## Decision

The visibility of `src/tools/cache.rs`, `src/tools/report.rs`, `src/tools/ui.rs`, and `src/semantic/assembly/model.rs` was modified to use `pub(crate) mod` instead of `pub mod`.

## Consequences

*   The public API surface is reduced and kept minimal.
*   Higher cohesion is achieved by ensuring internal structures do not leak out of their intended domains.
*   Internal tools and structural components are better encapsulated.
