# 16. Add Alchemist Tool

Date: 2026-03-14
Status: Proposed

## Context

The compiler's primary backend targets Rust. However, there is a need to prove the independence of the semantic phase from the Rust codegen phase, as well as a desire to have a dynamic target that aligns well with the "scripting" feel of small ΓΛΩΣΣΑ programs.

## Decision

We have introduced an experimental transpiler called "Alchemist" (`src/tools/alchemist.rs`) that converts ΓΛΩΣΣΑ programs into Python scripts.

## Consequences

### Positive
- **Backend Independence**: Proves that the semantic phase is truly independent of the Rust codegen phase.
- **Dynamic Target**: Offers a secondary, dynamic code generation target (Python) that can be useful for scripting and rapid prototyping.

### Negative
- **Maintenance Burden**: Introduces an additional backend that must be maintained and kept in sync with the primary semantic analysis phase and language features.
