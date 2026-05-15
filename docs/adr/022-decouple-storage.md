# 022. Decouple Storage from Core

Date: 2026-05-15

## Status

Proposed

## Context

Circular dependencies were causing build failures, and the core domain logic was tightly coupled to persistence concerns. This entanglement made it difficult to iterate quickly on business logic without pulling in database and filesystem dependencies.

## Decision

We decided to move the persistence logic out of the core module and into a dedicated `storage` crate. The core module now communicates with storage via a trait bound, rather than depending directly on concrete storage implementations.

## Consequences

- Build times improve due to the separation of concerns.
- Circular dependencies between core and storage are eliminated.
- FFI complexity increases as the interface between core and storage becomes more formalized.
