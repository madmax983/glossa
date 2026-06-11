# 021. Split Assembly Module

Date: 2026-04-21

## Status

Accepted

## Context

The `src/semantic/assembly.rs` file was a monolithic module mixing Data Transfer Objects (DTOs) like `AssembledStatement` and `Constituent` with complex semantic assembly logic (`Assembler`). This mixed responsibility made the file difficult to navigate, test, and maintain as the compiler's semantic phase grew in complexity.

## Decision

We split the monolithic `assembly.rs` file into a dedicated module directory `src/semantic/assembly/`.
The DTOs were extracted into `src/semantic/assembly/model.rs`, while the core assembly logic was moved to `src/semantic/assembly/mod.rs`.
Dependent modules were updated to import from `crate::semantic::assembly`.

## Consequences

- Improved separation of concerns between data and logic.
- Reduced file sizes, making the code easier to read and maintain.
- Simpler testing structure for assembly components.
- Existing imports had to be updated across internal tests and modules to reflect the new structure.
