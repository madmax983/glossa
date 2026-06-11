# 22. Encapsulate Internal Modules

Date: 2026-05-09

## Status

Accepted

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This violated the principle of encapsulation and created a sprawling public API, breaking encapsulation by exposing internal implementation details to the public API.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these modules with `pub(crate) mod`.

## Consequences

- **Positive:** Achieved higher cohesion by keeping the public API surface minimal.
- **Positive:** Ensured internal structures don't leak out of their intended domains.
- **Negative:** Internal modules can no longer be directly accessed from outside the crate.
