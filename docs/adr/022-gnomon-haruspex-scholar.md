# 022. Add Gnomon, Haruspex, and Scholar Tools

Date: 2026-04-22

## Status

Proposed

## Context

New experimental developer experience tools have been added to the `src/tools/` directory. Specifically, `gnomon` (a Big-O complexity estimator), `haruspex` (a Graphviz AST visualizer), and `scholar` (an API documentation generator). These tools expand the capabilities of the ΓΛΩΣΣΑ compiler but were introduced without formal architectural representation.

## Decision

We have added `Gnomon`, `Haruspex`, and `Scholar` as distinct modules within the `src/tools/` subsystem. They are categorized under the "Developer Experience (Nova)" boundary.

## Consequences

- **Positive:** Broadens the analytical and documentation capabilities of the compiler.
- **Positive:** Provides specific insights into program complexity (Gnomon), structure (Haruspex), and external interface (Scholar).
- **Negative:** Increases the footprint of the `tools` directory and requires updates to architectural diagrams to reflect the new boundaries.
