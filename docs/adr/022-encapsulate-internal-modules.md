# 022. Encapsulate Internal Modules

Date: 2026-06-13

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This violated the principle of encapsulation and created a sprawling public API, breaking internal encapsulation by exposing implementation details to the public API.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these internal modules using `pub(crate) mod`.
We also updated the associated architecture diagrams (`docs/architecture.md`) to explicitly label these internal components with `(Internal)`.

## Consequences

- Better encapsulation of the internal structure of the `tools` module and `semantic/assembly`.
- Minimized public API surface, ensuring internal structures do not leak outside of their intended domain.
- Requires developers to be conscious of module boundaries when adding utilities or using DTOs outside of their package structure.
