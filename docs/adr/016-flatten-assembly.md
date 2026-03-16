# 16. Flatten Assembly Module

Date: 2026-03-16
Status: Accepted

## Context

The `semantic` module previously contained an `assembly` directory with `mod.rs` and `model.rs` files. These files were tightly coupled via DTOs (`AssembledStatement`, `Constituent`, etc.). This directory nesting added architectural layers, cognitive overhead, and bloat without providing structural benefit, which violated the Razor persona's essentialism philosophy (KISS, YAGNI, DRY).

## Decision

We have merged `src/semantic/assembly/mod.rs` and `src/semantic/assembly/model.rs` into a single `src/semantic/assembly.rs` file.

## Consequences

### Positive
- **Reduced Bloat**: Eliminates unnecessary directory nesting and extra files, adhering to the principle of flattened hierarchies.
- **Improved Cohesion**: Tightly coupled Data Transfer Objects (DTOs) and the logic that uses them are now unified in a single file, reducing cognitive overhead and simplifying the module structure.
- **Consistency**: Aligns with similar flattening decisions made in the codebase, such as the `interpreter` module.

### Negative
- **File Size**: The combined `assembly.rs` file is larger than the individual `mod.rs` and `model.rs` files, which may slightly increase the time required to scroll through or conceptually navigate the single file.
