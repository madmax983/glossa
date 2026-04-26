# Decouple Storage from Core

## Status
Proposed

## Context
Circular dependencies were causing build failures, tangling the architecture and reducing cohesion between the core compiler logic and the persistent storage implementations.

## Decision
Move persistence logic to a dedicated crate (`storage`).

## Consequences
Build times improve, but FFI complexity increases.
