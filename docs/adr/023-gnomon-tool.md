# 023. Add Gnomon Complexity Estimator Tool

Date: 2026-05-16

## Status

Proposed

## Context

Developers need an automated way to evaluate the algorithmic time complexity of their code. While profiling tools measure actual runtime, there was no built-in tool to statically analyze and estimate Big-O complexity during the compilation phase, leaving developers to deduce loop nesting and branch depth manually.

## Decision

We have introduced "Gnomon" (`ὁ Γνώμων`) in `src/tools/gnomon.rs` as a static Big-O complexity estimator. Gnomon uses a visitor pattern over the semantic AST to track maximum loop nesting depth (`for` and `while` loops) and casts a shadow over the code to provide a heuristic complexity estimation.

## Consequences

*   **Positive:** Provides immediate feedback on code complexity to developers before runtime.
*   **Positive:** Expands the built-in diagnostic and teaching capabilities of the compiler.
*   **Negative:** As a static analysis tool relying on simple heuristics (like maximum loop depth), its estimations are approximations and may not reflect true runtime complexity in complex scenarios involving function calls or recursive structures.
