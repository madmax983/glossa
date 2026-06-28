# 022. Expand Nova Toolset

Date: 2026-06-28

## Status

Proposed

## Context

As the compiler evolved, several new experimental developer tools were added to `src/tools/` without formal architectural documentation:
- **Haruspex (ὁ Ἱεροσκόπος)**: Inspects the semantic AST and translates it into a Graphviz DOT graph for visualization.
- **Scholar (ὁ Σχολαστικός)**: Generates Markdown API documentation from the program's defined structures, traits, and functions.
- **Gnomon (ὁ Γνώμων)**: Estimates the Big-O time complexity of a program by statically analyzing loop depth in the semantic AST.

## Decision

We formally acknowledge the addition of these tools under the `nova` feature flag in `src/tools/`, alongside other Developer Experience (DX) tools. We have updated the architecture diagrams (C4 models) to reflect these new components and their dependencies on the Semantic Analyzer.

## Consequences

- **Architectural Transparency**: Developers can see how these tools fit into the compiler pipeline via the C4 Container diagram.
- **Encapsulation**: These tools remain gated behind the `nova` feature, preventing bloating of the core compiler binary.
- **Maintenance**: Future tools added to `src/tools/` must also be formally recorded and diagrammed.
