# 023. Add Gnomon Complexity Estimator

Date: 2026-05-09

## Status

Proposed

## Context

Understanding the performance implications of code can be challenging, particularly for loops and nested iterations. Developers using the ΓΛΩΣΣΑ compiler would benefit from automated insights into the algorithmic time complexity of their programs before they are executed.

## Decision

We have added "The Gnomon" (`ὁ Γνώμων`) to the compiler toolset (`src/tools/gnomon.rs`). This tool estimates the Big-O time complexity of a ΓΛΩΣΣΑ program by statically analyzing loop depth and structures within the semantic AST (`AnalyzedProgram`).

## Consequences

- **Performance Awareness:** Provides static analysis to estimate Big-O complexity, helping developers identify potentially slow algorithms (e.g., O(N^2) or worse) early.
- **Analysis Limitations:** As a static analysis tool, the estimations are based on structural loop depth and may not account for dynamic loop bounds or optimizations accurately.
