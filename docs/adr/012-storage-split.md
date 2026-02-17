# 12. Decouple Storage from Core

Date: 2025-05-23

## Status

Proposed

## Context

The compiler's core logic (`src/core`) was tightly coupled with file system operations and persistence logic (`src/storage`). This circular dependency caused several issues:
1.  **Build Failures**: Changes in storage often triggered full rebuilds of the core.
2.  **Testing Difficulty**: Unit testing core logic required mocking the entire file system.
3.  **Portability**: Porting the compiler to WASM (where `std::fs` is absent) was impossible due to the hard dependency.

## Decision

We will **decouple storage from core** by moving all persistence logic to a dedicated crate (or module acting as a crate) and defining a clear trait boundary.

*   `Core` will define a `Storage` trait.
*   `Storage` implementation will depend on `Core` (for types), but `Core` will not depend on the concrete `Storage` implementation.
*   The `CLI` or `Main` entry point will inject the concrete `Storage` into `Core`.

## Consequences

### Positive
*   **Build Times**: Core compilation is isolated from storage changes.
*   **Portability**: We can implement an `InMemoryStorage` for WASM/Web builds.
*   **Testing**: We can easily mock storage for core tests.

### Negative
*   **Complexity**: The dependency injection adds a layer of indirection.
*   **FFI**: Foreign Function Interface complexity increases as we cross crate boundaries.
