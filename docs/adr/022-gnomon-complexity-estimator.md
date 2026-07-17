# 022. Add Gnomon Complexity Estimator Tool

Date: 2026-07-17

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler translates Ancient Greek source code into rust. While analyzing the semantic AST, there was a need to estimate the execution time complexity of programs statically. Developers required a tool to measure the loop depth, similar to how a gnomon casts a shadow to indicate time.

## Decision

We have added the "Gnomon" (`ὁ Γνώμων`) tool to the "Developer Experience (Nova)" toolset (`src/tools/gnomon.rs`). Gnomon traverses the `AnalyzedStatement` AST to calculate the maximum loop nesting depth (`While` and `For` loops) and estimates the Big-O time complexity. It outputs these metrics using a `comfy-table` interface.

## Consequences

* **Positive:** Developers can quickly estimate the time complexity of their Ancient Greek programs statically.
* **Positive:** The tool adheres to the architectural pipeline by consuming the `AnalyzedProgram`.
* **Negative:** Adds to the maintenance burden of the tools ecosystem.
