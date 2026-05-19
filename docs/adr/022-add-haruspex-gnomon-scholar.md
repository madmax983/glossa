# 022. Add Haruspex, Gnomon, and Scholar Tools

Date: 2026-05-19

## Status

Proposed

## Context

As the compiler's semantic phase has grown in capability, developers and users needed specialized ways to inspect, estimate, and document the resulting `AnalyzedProgram`. Without proper visualization and static analysis tooling, debugging the raw AST and understanding API surfaces directly from the Ancient Greek source code was a challenging and opaque process.

## Decision

We have introduced three new Developer Experience tools within the `src/tools/` boundary:
*   **Haruspex (ὁ Ἱεροσκόπος):** An AST visualizer that translates the semantic tree structure into a Graphviz DOT representation.
*   **Gnomon (ὁ Γνώμων):** A Big-O complexity estimator that casts a shadow over the AST to statically analyze loop depth.
*   **Scholar (ὁ Σχολαστικός):** An API documentation generator that parses structures, traits, and functions to create GitHub-flavored Markdown.

## Consequences

*   **Positive:** Greatly improved visibility into the semantic AST, time complexity implications, and public API definitions.
*   **Positive:** Aligns with the "Living Maps" and "Documentation as Code" philosophies.
*   **Negative:** Adds to the maintenance burden of the `tools` module as the AST and semantic model continue to evolve.
