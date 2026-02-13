# 10. Decouple Storage from Core

Date: 2025-05-20

## Status

Proposed

## Context

Circular dependencies between the core compiler modules and the persistence logic were causing build failures and making it difficult to test components in isolation. The `storage` logic was tightly coupled with `semantic` analysis, leading to a monolithic structure.

## Decision

We will move all persistence and storage logic to a dedicated `storage` module (or crate).

1.  **Create `src/storage/`:** A new module for persistence.
2.  **Decouple:** The core compiler will interact with storage via a trait boundary, removing the direct dependency.

## Consequences

*   **Build Times:** Compilation time should improve due to better parallelism.
*   **Complexity:** FFI complexity might increase if we extract this to a separate crate later.
*   **Testing:** Storage logic can be tested independently of the compiler core.
