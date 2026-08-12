# 022. Add Gnomon to Developer Experience Tools

Date: 2026-08-12

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler includes the "Gnomon" tool (`src/tools/gnomon.rs`), which estimates the Big-O time complexity of a ΓΛΩΣΣΑ program by statically analyzing loop depth in the semantic AST.
However, this tool is missing formal architectural documentation and representation in our architectural diagrams, which violates the Codex principles of Architectural Transparency.

## Decision

We formally recognize the **Gnomon** tool as a component within the "Developer Experience (Nova)" container (`src/tools/`).

Gnomon is defined as the Big-O complexity estimator tool that takes an `AnalyzedProgram` from the Semantic Analyzer and calculates the maximum depth of loop nesting (`for` and `while` loops). It will be integrated into the architecture diagram (`docs/architecture.md`) alongside other Nova tools.

## Consequences

*   **Positive:** Architectural Clarity: The role and existence of the Gnomon tool are explicitly documented and mapped, eliminating implicit decisions.
*   **Positive:** Enhanced Visibility: Users and contributors can more easily discover and understand the purpose of the complexity estimation feature.
*   **Negative:** Maintenance Surface: As a recognized tool, Gnomon must be maintained to keep pace with changes to the semantic model (`AnalyzedProgram`). Updates to the control flow logic in the AST or HIR will require corresponding updates to the Gnomon tool's visitor pattern to ensure accurate estimation.
