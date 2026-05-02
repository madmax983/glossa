# 022. Encapsulate Internal Modules

Date: 2026-05-01

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were previously exposed as `pub mod`. This broke encapsulation by unnecessarily exposing internal implementation details and utilities to the public API of the compiler crate, making the API surface larger and more confusing for consumers.

## Decision

We have modified the module declarations in `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these internal modules using `pub(crate) mod`.

## Consequences

- **Positive:** Achieves higher cohesion by keeping the public API surface minimal.
- **Positive:** Ensures internal data structures and utilities do not leak out of their intended domains, making the system easier to maintain.
- **Negative:** Internal modules can no longer be accessed outside the crate (e.g., from external tests or binaries), which enforces strict boundaries but requires using the correct public APIs.
