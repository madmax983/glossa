# 10. Decouple Storage from Core

Date: 2025-02-12

## Status

Proposed

## Context

Circular dependencies were causing build failures and tight coupling between core business logic and persistence mechanisms. The core system was attempting to import storage implementations, while storage needed to know about core entities, creating a cycle.

## Decision

Move persistence logic to a dedicated crate or module named `storage`. The Core will define traits (interfaces) that Storage implements, inverting the dependency.

## Consequences

*   **Positive:** Build times improve due to reduced recompilation scope.
*   **Positive:** Core logic becomes testable without a database.
*   **Negative:** FFI complexity increases if we need to expose storage across language boundaries.
