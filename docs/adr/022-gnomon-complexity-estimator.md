# 022. Gnomon Complexity Estimator

Date: 2026-06-30
Status: Accepted

## Context

As ΓΛΩΣΣΑ programs grow in complexity, developers need a way to understand the performance implications of their code statically. We needed a tool to estimate the Big-O time complexity by analyzing loop depth directly from the semantic AST.

## Decision

We introduced the "Gnomon" (ὁ Γνώμων) tool in `src/tools/gnomon.rs`. It acts as a visitor that traverses the Abstract Syntax Tree to calculate loop depth, estimating the execution time complexity. It was also recently refactored to use a flat procedural function approach to eliminate OOP overhead.

## Consequences

- **Positive:** Developers can quickly estimate algorithmic complexity without running the code.
- **Positive:** Fits perfectly into the existing Developer Experience (Nova) toolset.
- **Negative:** Adds complexity to the compiler's tool suite, requiring maintenance of the complexity estimation logic as the AST evolves.
