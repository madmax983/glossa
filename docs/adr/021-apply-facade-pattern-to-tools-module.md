# 21. Apply Facade Pattern to Tools Module

Date: 2026-04-22

## Status

Accepted

## Context

The `src/tools/mod.rs` module historically exported many of its internal submodules directly as `pub mod`. As new tools like Papyrus, Mentor, Weave, Labyrinth, and Catalog were added to the "Developer Experience (Nova)" toolset, the public API of the `glossa::tools` module grew increasingly sprawling and unstructured.
This "leaky abstraction" forced consumers (such as the CLI, REPL, or integration tests) to know the exact internal module paths to import specific tools (e.g., `use glossa::tools::papyrus::run_papyrus;` instead of `use glossa::tools::run_papyrus;`). This tight coupling made refactoring internal tool structures difficult without breaking downstream consumers and violated architectural encapsulation.

## Decision

We have applied the **Facade pattern** to the `src/tools/mod.rs` module.
All internal tool modules (e.g., `papyrus`, `interpreter`, `mosaic`, `tester`) have been restricted to `pub(crate) mod` visibility. The `tools` module now acts as a single, cohesive Facade by explicitly re-exporting only the intended public functions, structs, and enums (e.g., `run_papyrus`, `run_tests`, `run_file`, `Interpreter`) using `pub use` statements at the top level.

## Consequences

*   **Positive:** The `glossa::tools` API is now strictly defined, smaller, and easier for external consumers to understand and use.
*   **Positive:** Downstream dependencies (CLI, tests) are decoupled from the internal file hierarchy of the `tools` module, allowing for easier future refactoring.
*   **Positive:** Enforces stronger architectural boundaries and encapsulation (hiding internal implementation details).
*   **Negative:** Developers adding new tools must remember to manually export their public APIs in `src/tools/mod.rs`, creating a minor friction point.
