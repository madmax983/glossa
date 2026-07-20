# 022. Split Conversion Module

Date: 2026-05-15

## Status

Proposed

## Context

The `src/semantic/conversion.rs` module had grown into a monolithic file (the "Blob" anti-pattern), mixing extraction logic, classification logic, and tests. This made it difficult to navigate and maintain.

## Decision

The monolithic `conversion.rs` file was split into a dedicated module directory `src/semantic/conversion/`. Extraction logic was moved to `extract.rs`, classification logic to `classify.rs`, and tests to `tests.rs`. A new `mod.rs` acts as the module entry point, maintaining visibility boundaries with `pub(crate)`.

## Consequences

- File sizes are significantly reduced, improving separation of concerns.
- Strict encapsulation is maintained.
