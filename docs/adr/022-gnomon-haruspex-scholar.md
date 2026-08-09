# 022. Introduce Gnomon, Haruspex, and Scholar

Date: 2026-08-09

## Status

Proposed

## Context

The Developer Experience (Nova) tool suite requires more diagnostic and documentation tools. We need a way to estimate Big-O complexity (Gnomon), visualize the semantic AST as a graph (Haruspex), and generate API documentation (Scholar).

## Decision

We introduce three new tools to `src/tools/`:
- **Gnomon**: Estimates the Big-O time complexity by statically analyzing loop depth in the semantic AST.
- **Haruspex**: Translates the semantic AST into a DOT graph for Graphviz visualization.
- **Scholar**: Parses a program and automatically generates comprehensive Markdown API documentation.
These tools are integrated into the Developer Experience (Nova) tool suite and consume the `AnalyzedProgram`.

## Consequences

- Enhances the developer experience by providing more insights and automation.
- Requires maintenance of these new tools and integration with the existing compiler pipeline.
- Modifies the architectural boundaries and relationships within the `tools` module.
