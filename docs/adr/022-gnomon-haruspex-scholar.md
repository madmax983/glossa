# 022. Add Gnomon, Haruspex, and Scholar to Developer Experience Tools

Date: 2026-08-18

## Status

Proposed

## Context

The `src/tools/` directory has accumulated three new experimental Developer Experience (Nova) tools without accompanying Architectural Decision Records: `gnomon` (Big-O Complexity Estimator), `haruspex` (Graphviz AST Visualizer), and `scholar` (API Doc Generator). To maintain Architectural Transparency, these tools must be formally documented as part of the ecosystem.

## Decision

We formally include `gnomon`, `haruspex`, and `scholar` as experimental tools within the "Developer Experience (Nova)" toolset.

## Consequences

- Better insights into AST visualization, algorithm complexity, and API documentation for compiler developers and language users.
- Increased maintenance burden as these tools must be kept compatible with the evolving Semantic Analyzer API.
