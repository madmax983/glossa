# 022. Add Gnomon (ὁ Γνώμων) Big-O Complexity Estimator

Date: 2026-08-19
Status: Proposed

## Context
As programs written in ΓΛΩΣΣΑ become more complex, developers need tools to understand their performance characteristics statically. The "Gnomon" tool (`src/tools/gnomon.rs`) was introduced to estimate the Big-O time complexity by statically analyzing loop depth (e.g., `while` and `for` loops) in the semantic AST. However, this addition to the "Developer Experience (Nova)" toolset is currently unrecorded in our architecture decision records, violating the principle of Architectural Transparency.

## Decision
We formally recognize the **Gnomon** tool as a component within the "Developer Experience (Nova)" container in `src/tools/`.
Gnomon is defined as the Big-O complexity estimator that takes an `AnalyzedProgram` from the Semantic Analyzer, traverses its abstract syntax tree to calculate loop depth, and reports an estimated time complexity (e.g., O(1), O(N), O(N^2)).

## Consequences
- **Positive:** Improved architectural clarity. Developers have a static analysis tool for estimating program complexity without running code.
- **Negative:** Increased maintenance surface. Changes to control flow structures in the semantic AST will require updates to the Gnomon visitor logic.
