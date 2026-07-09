# Decouple Storage from Core

Date: 2024-10-24

## Status

Proposed

## Context

Circular dependencies were causing build failures, and the `storage` module was deeply intertwined with the `core` logic. This made the codebase harder to maintain and test, and increased compile times significantly.

## Decision

Move persistence logic to a dedicated crate or isolated module (`storage`), decoupling it from the `core` components. The `Core` module now relies on `Storage` via Trait Bounds, breaking the circular dependency.

## Consequences

Build times improve, and the system is more modular, but FFI or trait-based complexity increases slightly to support the decoupled boundary.
