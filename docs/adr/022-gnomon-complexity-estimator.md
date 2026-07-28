# 022. Add Gnomon Complexity Estimator Tool

Date: 2026-07-28

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler currently provides various developer experience tools (like the Alchemist, Weave, and Papyrus) that operate on the semantic AST. However, there has been a gap in providing developers with insights into the performance characteristics of their Ancient Greek code. Specifically, there is a need to estimate the Big-O time complexity to help developers identify potentially slow algorithms and nested loops statically, before runtime.

## Decision

We have added "Gnomon" (`Γνώμων`) to the "Developer Experience (Nova)" toolset (`src/tools/gnomon.rs`). Gnomon acts as a `Visitor` over the `AnalyzedProgram` from the Semantic Analyzer. It statically analyzes loop depth (by tracking the nesting of `While` and `For` statements) to cast a "shadow" over the program's AST and estimate its execution time complexity (e.g., O(1), O(N), O(N^2)). The results are formatted into a terminal table for the developer.

## Consequences

* **Positive:** Developers receive immediate, static feedback on the algorithmic complexity of their code, encouraging better performance practices.
* **Positive:** Integrates cleanly as a consumer of the `AnalyzedProgram`, adhering to our existing tool pipeline architecture without modifying core analysis phases.
* **Negative:** The complexity estimation is purely structural (loop depth) and may not perfectly reflect true runtime complexity if there are early exits (`break`, `return`) or complex inner loop bounds.
