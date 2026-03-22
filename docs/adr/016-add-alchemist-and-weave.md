# 16. Add Alchemist and Weave Tools

Date: 2024-11-20

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler's "Nova" experimental feature set aims to explore different avenues for development experience and utility. Two new features have been introduced:
1. **The Alchemist (`src/tools/alchemist.rs`)**: An experimental Python transpiler. This serves as a proof-of-concept to show that the semantic analysis phase is entirely decoupled from the Rust codegen backend, allowing for alternative output languages.
2. **Weave (`src/tools/weave.rs`)**: A documentation generator that outputs a 'Rosetta Stone' Markdown document. It combines the original Greek source code, the intermediate Semantic Assembly (Mosaic) steps, and the final generated Rust code into a single, highly educational artifact.

These components need to be formally recognized in the system architecture to maintain structural transparency.

## Decision

We have decided to formalize "The Alchemist" and "Weave" as official tools within the "Developer Experience (Nova)" boundary of the compiler pipeline.

*   **The Alchemist** is tasked with converting the `AnalyzedProgram` from the semantic phase into Python source code.
*   **Weave** is tasked with orchestrating the parser, semantic analyzer, Rust code generator, and the Mosaic tool to assemble a unified documentation output.

## Consequences

*   **Architectural Visibility:** The system context and C4 Container diagrams will be updated to reflect the presence of `alchemist` and `weave` inside the `tools` boundary, downstream from `semantic`.
*   **Decoupling Verification:** The successful implementation of the Alchemist confirms our architectural goal that `glossa::semantic` produces a generic HIR (High-Level Intermediate Representation) suitable for multiple backends.
*   **Educational Value:** Weave provides a concrete, automated way to generate educational materials, reinforcing ΓΛΩΣΣΑ's goal of teaching programming concepts through the lens of ancient languages.
*   **Maintenance Burden:** We now maintain an experimental Python backend alongside the primary Rust backend. As the language grows, ensuring feature parity (or at least graceful degradation) in the Alchemist will require additional effort.
