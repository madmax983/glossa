# 022. Encapsulate Internal Modules

Date: 2024-06-14

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`, breaking encapsulation by exposing internal implementation details to the public API. Atlas noticed this tangle and recommended encapsulating these internal details.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these internal modules (`cache`, `report`, `ui`, `model`) with `pub(crate) mod` visibility.

## Consequences

- Achieved higher cohesion by keeping the public API surface minimal.
- Ensured internal structures do not leak out of their intended domains.
- Requires changes to our C4 diagrams to indicate these components are strictly internal.
