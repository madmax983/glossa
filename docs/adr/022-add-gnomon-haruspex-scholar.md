# 022. Add Gnomon, Haruspex, and Scholar Tools

Date: 2026-05-10

## Status

Proposed

## Context

The `glossa` compiler has been expanded with three new tools: "The Gnomon", "The Haruspex", and "The Scholar". These were added to `src/tools/` but their architectural integration was not formally documented.
- **The Haruspex** (`src/tools/haruspex.rs`) inspects the semantic AST and translates it into a DOT graph for visualization.
- **The Gnomon** (`src/tools/gnomon.rs`) estimates the Big-O time complexity of a program by statically analyzing loop depth.
- **The Scholar** (`src/tools/scholar.rs`) generates comprehensive Markdown API documentation (`doc.md`) from a program.

## Decision

We are formally integrating these tools into the developer experience suite (Nova). They operate on the output of the Semantic Analyzer (`Analyzed Program`). This ADR records their existence and updates the system documentation to reflect their boundaries.

## Consequences

- **Positive:** Improved observability of semantic trees via Haruspex.
- **Positive:** Better understanding of performance bottlenecks via Gnomon.
- **Positive:** Automated generation of API docs bridging the semantic tree to human-readable markdown.
- **Negative:** Increased maintenance surface area in `src/tools`.
