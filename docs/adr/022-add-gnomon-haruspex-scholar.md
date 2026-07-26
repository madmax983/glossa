# 022. Add Gnomon, Haruspex, and Scholar Tools

Date: 2026-07-26

## Status

Proposed

## Context

As the ΓΛΩΣΣΑ compiler evolves, there is a growing need for advanced tools to support developer experience (DX). We identified the need to:
1. Estimate the Big-O time complexity of Glossa programs statically.
2. Visually inspect the raw semantic Abstract Syntax Tree (AST).
3. Automatically generate API documentation for Glossa libraries.

## Decision

We have added three new tools to the Developer Experience (Nova) toolset in `src/tools/`:
1. **The Gnomon (`ὁ Γνώμων`) - `gnomon.rs`**: A Big-O complexity estimator that traverses the semantic AST to calculate loop depth.
2. **The Haruspex (`ὁ Ἱεροσκόπος`) - `haruspex.rs`**: A Graphviz AST Visualizer that translates the semantic AST into a DOT graph.
3. **The Scholar (`ὁ Σχολαστικός`) - `scholar.rs`**: An API documentation generator that parses a program and extracts definitions into Markdown.

## Consequences

* **Positive**: Enhances the Nova DX by giving developers powerful profiling, visualization, and documentation tools.
* **Positive**: Continues the architectural pattern of isolating specific user-facing capabilities in dedicated `src/tools` modules.
* **Negative**: Increases the surface area of the codebase, meaning these tools must be updated whenever the core semantic AST changes.
