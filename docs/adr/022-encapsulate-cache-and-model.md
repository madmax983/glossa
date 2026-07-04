# 022. Encapsulate Cache and Model Modules

Date: 2026-07-04

## Status

Proposed

## Context

Internal modules within `src/tools/` (specifically `cache`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by exposing internal implementation details to the public API, which can lead to brittle dependencies and a sprawling, hard-to-maintain public interface.

## Decision

We modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these internal modules by declaring them with `pub(crate) mod` instead of `pub mod`.

## Consequences

- **Positive:** We achieve higher cohesion by keeping the public API surface minimal.
- **Positive:** Internal structures and implementation details no longer leak out of their intended domains.
- **Negative:** Developers must be conscious of module boundaries when adding new internal utilities; they cannot simply rely on everything being publicly available across the workspace.
