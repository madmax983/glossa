# 20. Enforce Facade Pattern in Tools Module

Date: 2024-05-21

## Status

Proposed

## Context

The `src/tools/mod.rs` module historically exposed many of its internal submodules (such as `cli`, `dictionary`, `tester`, `runner`, `repl`, `narrator`, `alchemist`, and `auditor`) directly to the rest of the application using `pub mod`. This violated the principle of encapsulation, leaking internal implementation details and cluttering the public API. It made it difficult for developers to determine the intended public interface of the tools suite versus its internal machinery. An earlier ADR (016) addressed `ui` and `report`, but this needs to be applied universally across all internal tools.

## Decision

We have applied the Facade pattern to `src/tools/mod.rs`. The visibility of the internal tool submodules has been restricted from `pub mod` to `pub(crate) mod` (with `highlight` being an exception as it is needed publicly in some contexts). To ensure that external consumers (like `src/main.rs`, doc tests, and integration tests) still function correctly without being coupled to internal module structures, we have added explicit `pub use` statements in `src/tools/mod.rs` for the specifically required items (e.g., `run_file`, `Cli`, `Commands`).

## Consequences

*   **Positive:** The internal structure of the `tools` module is strictly encapsulated, hiding implementation details from the rest of the application.
*   **Positive:** The public API of the `glossa::tools` crate is clean, flattened, and intentionally designed via the Facade pattern.
*   **Negative:** Developers must remember to add a `pub use` statement in `src/tools/mod.rs` if a newly developed tool needs to be accessed by the CLI binary or external integration tests.
