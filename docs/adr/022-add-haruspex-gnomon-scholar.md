# 022. Add Haruspex, Gnomon, and Scholar Tools

Date: 2026-06-08

## Status

Proposed

## Context

Recently, new tools were added to the `src/tools/` directory to enhance the developer experience (Nova) of the ΓΛΩΣΣΑ compiler. These include:
- `haruspex` (`src/tools/haruspex.rs`): Translates the semantic AST into a Graphviz DOT graph.
- `gnomon` (`src/tools/gnomon.rs`): Estimates Big-O time complexity by analyzing loop depths.
- `scholar` (`src/tools/scholar.rs`): Generates comprehensive Markdown API documentation (`doc.md`) from ΓΛΩΣΣΑ source.

These tools were added to the codebase but were not documented in an Architecture Decision Record (ADR) or the `docs/architecture.md` diagram, violating our architectural transparency standards (Codex).

## Decision

We document the addition of `haruspex`, `gnomon`, and `scholar` as first-class tools within the `src/tools/` directory. The architecture diagrams in `docs/architecture.md` must be updated to reflect their existence as consumers of the `AnalyzedProgram` from the Semantic Analyzer.

## Consequences

- **Transparency:** The architecture diagram now accurately reflects the current state of the tooling ecosystem.
- **Maintenance:** These tools are officially recognized and subject to the same standards as other `tools/` modules.
