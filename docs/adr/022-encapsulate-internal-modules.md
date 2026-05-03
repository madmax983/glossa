# 022. Encapsulate Internal Modules

Date: 2026-05-03

## Status

Accepted

## Context

Several utility and data-transfer modules within the compiler were unnecessarily exposed to the public API via `pub mod`. This included internal developer tools (`src/tools/cache.rs`, `src/tools/report.rs`, and `src/tools/ui.rs`) as well as internal data structures used during semantic assembly (`src/semantic/assembly/model.rs`). This sprawling public footprint violated encapsulation principles, increasing the risk of external dependencies on internal implementation details and creating a poorly-defined boundary for the system's public API.

## Decision

We restricted the visibility of these modules to their intended domains by changing their declarations to `pub(crate) mod`.
Specifically:
- In `src/tools/mod.rs`, the `cache`, `report`, and `ui` modules were marked as `pub(crate) mod`.
- In `src/semantic/assembly/mod.rs`, the `model` module was marked as `pub(crate) mod`.

## Consequences

- **Positive:** Higher cohesion and a tighter boundary. Internal helpers and DTOs no longer leak into the public API of the crate.
- **Positive:** Developers can refactor internal modules like `report` and `ui` with confidence that changes won't break external consumers.
- **Negative:** Other modules within the workspace must correctly import these components via internal paths, requiring some internal import adjustments when this change was made.
