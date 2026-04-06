# Decouple Storage from Core

**Status:** Proposed

## Context
Circular dependencies were causing build failures.

## Decision
Move persistence logic to a dedicated crate.

## Consequences
Build times improve, but FFI complexity increases.
