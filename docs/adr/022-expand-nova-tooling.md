# 022. Expand Nova Tooling with Haruspex, Gnomon, and Scholar

Date: 2026-05-31

## Status

Proposed

## Context

As the ΓΛΩΣΣΑ compiler evolves, there is a growing need for deeper insights into the semantic AST, time complexity, and API documentation. Developers require tools to visualize the exact structure of the AST, estimate performance characteristics statically, and automatically generate API documentation for their ΓΛΩΣΣΑ programs.

## Decision

We have added three new experimental tools to the Nova Developer Experience suite, located in `src/tools/`:
- **Haruspex (`src/tools/haruspex.rs`)**: A Graphviz AST Visualizer that translates the semantic AST into a DOT graph for visual inspection.
- **Gnomon (`src/tools/gnomon.rs`)**: A Big-O Complexity Estimator that statically analyzes loop depth in the AST to estimate execution time complexity.
- **Scholar (`src/tools/scholar.rs`)**: An API Doc Generator that distills structural components (structs, traits, and functions) into GitHub-flavored Markdown.

## Consequences

- Improved visibility into the raw semantic tree structure with Haruspex.
- Early insights into potential performance bottlenecks with Gnomon.
- Automated API documentation generation with Scholar, acting as a bridge between semantic analysis and human-readable references.
- Increased tooling footprint in the Nova suite.
