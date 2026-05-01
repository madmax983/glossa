# 022. Expand Nova Toolset

Date: 2026-05-01

## Status

Proposed

## Context

The Developer Experience (Nova) toolset was expanded with three new capabilities directly analyzing the semantic AST:
1. **Gnomon (ὁ Γνώμων):** Estimates Big-O execution time complexity by statistically analyzing loop depth in the AST.
2. **Haruspex (ὁ Ἱεροσκόπος):** Inspects the raw `AnalyzedProgram` and translates it into a GraphViz DOT graph for debugging compiler internals.
3. **Scholar (ὁ Σχολαστικός):** Generates Markdown API documentation for a ΓΛΩΣΣΑ program's types, traits, and functions.

## Decision

These tools have been added to the `src/tools/` directory (`gnomon.rs`, `haruspex.rs`, and `scholar.rs`) alongside existing Developer Experience tools like Labyrinth and Weave.

## Consequences

- Improves compiler transparency and user documentation generation.
- The `semantic` module acts as the core provider of `AnalyzedProgram` representations to an increasing number of Nova tools, solidifying the AST as the primary interaction boundary.
- Visualizations from Haruspex require `dot` to be installed on the host system to render the graphs.
