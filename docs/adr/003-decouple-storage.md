# 3. Decouple Storage from Core

Date: 2024-10-25
Status: Proposed

## Context

Circular dependencies were causing build failures and making the codebase difficult to maintain. The Core module depended on Storage for persistence, while Storage depended on Core for data structures.

## Decision

Move persistence logic to a dedicated crate. The Storage crate will be independent of Core, or depend on it only via interfaces/traits defined in a common location if necessary.

## Consequences

- **Build times improve:** Parallel compilation is enabled.
- **FFI complexity increases:** Crossing the crate boundary might require more complex FFI or serialization.
- **Clearer boundaries:** Enforces separation of concerns.
