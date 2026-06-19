# 022. Encapsulate Internal Modules

Date: 2026-06-19

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by unnecessarily exposing internal implementation details to the public API of the compiler.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these internal modules with `pub(crate) mod`.

## Consequences

- **Positive:** Achieved higher cohesion by keeping the public API surface minimal.
- **Positive:** Ensures internal structures and DTOs don't leak out of their intended domains.
- **Negative:** Internal modules can no longer be directly accessed by external crates or integration tests, requiring them to use the defined public interfaces.
