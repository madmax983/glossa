# 22. Expand Nova Tool Suite

Date: 2024-11-20

## Status

Accepted

## Context

As the ΓΛΩΣΣΑ compiler ecosystem matures, developers require deeper insights into their code's performance, structure, and usage. The existing Nova tools provided baseline capabilities, but lacked automated documentation generation, raw AST visualization, and complexity estimation.

## Decision

We have formally added three new tools to the Nova Developer Experience suite:
- **The Scholar (ὁ Σχολαστικός)** (`scholar.rs`): Generates Markdown API documentation (`doc.md`) from the AST.
- **The Haruspex (ὁ Ἱεροσκόπος)** (`haruspex.rs`): Translates the semantic AST into a Graphviz DOT format for raw tree visualization.
- **The Gnomon (ὁ Γνώμων)** (`gnomon.rs`): Statically analyzes loop depth to estimate Big-O time complexity.

## Consequences

- **Positive:** Improved developer experience with built-in API doc generation, structural visualization, and static complexity analysis.
- **Negative:** Increased surface area of the compiler's `tools` module and a slight increase in binary size when the `nova` feature is enabled.
