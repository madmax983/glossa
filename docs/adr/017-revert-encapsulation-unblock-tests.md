# 17. Revert Encapsulation of Internal Modules to Unblock Tests

Date: 2026-04-08

## Status

Accepted

## Context

Previously, we strictly encapsulated our internal modules by using `pub(crate) mod` to prevent leaking implementation details into the public API (as noted in ADR 016). While this successfully created a clean boundary for the crate, it severely hindered our testing infrastructure.

Because the Rust `tests/` directory operates as an external consumer of the `glossa` crate, integration tests located there can only access items that are explicitly declared as `pub`. By sealing off internal modules like `src/tools/report.rs` or `src/tools/ui.rs`, we inadvertently broke critical test coverage. Deeply nested tests or specialized testing tools were unable to import the structures they needed to verify internal compiler logic or perform isolated component testing. Refactoring all tests to live inside `src/` was deemed too invasive and convoluted.

## Decision

We have decided to partially revert the strict encapsulation of internal modules. Modules that require external testing or are imported by the test suite (such as `cache`, `report`, and `ui`) have been restored to `pub mod`. We rely on convention and documentation rather than Rust's visibility system to guide users towards the intended public API.

## Consequences

* **Positive:** The external test suite in `tests/` can now successfully import and verify internal components.
* **Positive:** Unblocks development and stabilizes CI test pipelines without requiring massive restructuring of the testing architecture.
* **Negative:** The crate's public API is once again sprawling and polluted with internal implementation details, which may confuse end users about what constitutes the stable compiler API.
