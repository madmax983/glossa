# 022. Encapsulate Internal Modules

Date: 2026-07-18

## Status

Proposed

## Context

Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`. This broke encapsulation by exposing internal implementation details to the public API. It created a sprawling public API and made it difficult to discern which modules were intended for external consumption versus which were strictly internal helpers.

## Decision

We have updated the visibility of internal modules to restrict them with `pub(crate) mod`.
Specifically, we modified `src/tools/mod.rs` to make `cache`, `report`, and `ui` internal, and `src/semantic/assembly/mod.rs` to make `model` internal.

## Consequences

*   **Positive:** Achieved higher cohesion by keeping the public API surface minimal.
*   **Positive:** Ensures internal structures don't leak out of their intended domains.
*   **Negative:** Developers must be conscious of module boundaries when adding new internal utilities.

## Visuals

```mermaid
classDiagram
  namespace Tools {
    class Cache { <<internal>> }
    class Report { <<internal>> }
    class UI { <<internal>> }
    class Runner
    class CLI
  }
  namespace Assembly {
    class Assembler
    class Model { <<internal>> }
  }
  Assembler --> Model : Uses
  Tools --> Assembly : Analyzes
```
