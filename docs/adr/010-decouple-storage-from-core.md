# 10. Decouple Storage from Core

Date: 2024-11-20

## Status

Proposed

## Context

The `Core` module of the ΓΛΩΣΣΑ compiler, responsible for parsing and semantic analysis, was directly depending on persistence logic (e.g., file caching, artifact management) located in `src/tools/cache.rs` and potentially other utility modules. This coupling created several issues:

1.  **Circular Dependencies:** As the compiler grew, `Core` needed to verify artifacts, but `Storage` needed `Core`'s types to serialize them.
2.  **Testing Difficulty:** Unit testing `Core` required mocking file system interactions, which was cumbersome.
3.  **Build Times:** Changes in the storage layer forced recompilation of the entire core logic.

## Decision

We will decouple the persistence logic into a dedicated `Storage` component (currently implemented via `src/tools/cache.rs` and related utilities).

The `Core` module will interact with `Storage` exclusively through trait bounds or clean interfaces, rather than direct dependencies on concrete implementations where possible. This aligns with the Hexagonal Architecture (Ports and Adapters) pattern.

## Consequences

### Positive

*   **Improved Build Times:** Changes to the storage implementation will not necessarily trigger a full recompilation of `Core` if the interface remains stable.
*   **Testability:** `Core` can be tested with in-memory storage mocks, isolating it from file system side effects.
*   **Cleaner Architecture:** Clear separation of concerns makes the codebase easier to reason about.

### Negative

*   **Increased Complexity:** Introducing abstraction layers (traits) adds some boilerplate and indirection.
*   **Interface Management:** We must carefully design the storage interface to be flexible enough for future needs without leaking implementation details.
