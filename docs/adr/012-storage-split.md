# 12. Decouple Storage from Core

Date: 2023-10-25

## Status

Proposed

## Context

Circular dependencies were causing build failures between the core system and the storage logic. The tight coupling made it difficult to iterate on the core without recompiling the entire storage subsystem, leading to decreased developer velocity and fragile boundaries.

## Decision

Move persistence logic to a dedicated `storage` crate. This enforces a strict one-way dependency where `Core` uses `Storage` via trait bounds, decoupling the implementations.

## Consequences

Build times improve, but FFI complexity increases. We now have a clearer architectural boundary, preventing future circular dependencies, though we must carefully manage trait definitions in the core to allow the storage implementation to hook in properly.
