# 16. Enforce Tool Encapsulation

Date: 2024-05-18

## Status

Accepted

## Context

The `src/tools/` directory contained many internal tools (like `ui` and `report`) that were needlessly exposed via `pub mod` to the rest of the application. This violated the principle of encapsulation and created a sprawling public API, making it difficult to discern which tools were intended for external consumption (e.g., via the CLI or integration tests) versus which were strictly internal helpers.

## Decision

We have updated the visibility of internal tool modules in `src/tools/mod.rs`. Specifically, the `report` and `ui` modules have been changed from `pub mod` to `pub(crate) mod`. Modules that are explicitly used by the main binary, re-exported by `src/lib.rs`, or required by external integration tests have been kept as `pub mod` to ensure compilation succeeds while strictly defining the tool suite's public boundary.
We also resolved an associated `dead_code` warning on the unused `start` function in `ui.rs` by attributing it with `#[allow(dead_code)]`.

## Consequences

*   **Positive:** The internal structure of the `tools` module is now better encapsulated.
*   **Positive:** The public API of the `glossa` crate's tool suite is smaller and more intentional.
*   **Negative:** Developers must be conscious of module boundaries when adding new internal utilities; they cannot simply rely on everything being publicly available across the workspace.
