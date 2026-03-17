# 16. Flatten Semantic Assembly Module

Date: 2026-03-17
Status: Accepted

## Context

The `src/semantic/assembly/mod.rs` and `src/semantic/assembly/model.rs` files were tightly coupled via DTOs (`AssembledStatement`, `Constituent`, etc.) and introduced unnecessary directory nesting within the `semantic` module. This structure violated the Razor persona's essentialism philosophy (KISS, YAGNI, DRY) by adding architectural layers and cognitive overhead without providing clear structural benefits.

## Decision

We eliminated the unnecessary directory nesting by flattening the semantic assembly module. The separate files `src/semantic/assembly/mod.rs` and `src/semantic/assembly/model.rs` have been merged into a single `src/semantic/assembly.rs` file. Types previously nested in `model::` must now be accessed directly via `crate::semantic::assembly::` to prevent path resolution errors in dependent tools.

## Consequences

### Positive
- **Reduced Bloat**: Eliminates the `src/semantic/assembly` directory, adhering to the principle of flattened hierarchies.
- **Cognitive Load**: Reduces architectural layers and cognitive overhead without altering compiler behavior.

### Negative
- **File Size**: The combined `src/semantic/assembly.rs` file is larger, which might marginally increase navigation time within that specific file.
