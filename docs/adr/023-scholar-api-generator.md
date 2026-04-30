# 023. Add Scholar API Doc Generator

Date: 2026-04-30

## Status

Accepted

## Context

As developers build larger programs and libraries in ΓΛΩΣΣΑ, there is a growing need to document the APIs (structs, traits, and functions) clearly and automatically. Hand-writing documentation is error-prone and falls out of sync with the codebase. We need a way to generate structured API documentation directly from the analyzed source code.

## Decision

We have added the "Scholar" (ὁ Σχολαστικός) tool to the Developer Experience toolset (`src/tools/scholar.rs`).
The Scholar consumes the `AnalyzedProgram`'s scope to extract definitions for Types (Structs), Traits, and Functions, and outputs Markdown documentation formatting these structures.

## Consequences

*   **Positive:** Developers can automatically generate Markdown documentation for their ΓΛΩΣΣΑ APIs.
*   **Positive:** Consistency is improved, as the documentation always reflects the actual parsed structures.
*   **Negative:** Adds maintenance burden for another tool in the `src/tools/` directory.
