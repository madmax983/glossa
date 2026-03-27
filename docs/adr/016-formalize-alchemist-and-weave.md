# 16. Formalize Alchemist and Weave Tools

Date: 2025-02-25
Status: Proposed

## Context

The `src/tools/` directory contains two new modules, `alchemist.rs` and `weave.rs`, which lack formal documentation and architectural recognition.

`alchemist.rs` (The Alchemist) transpiles ΓΛΩΣΣΑ programs to Python scripts. This demonstrates the independence of the semantic analysis phase from the Rust code generation phase, offering a dynamic script target for small programs.

`weave.rs` (The Weave) generates a "Rosetta Stone" Markdown document. This exporter combines ΓΛΩΣΣΑ source code, the semantic assembly visualization (Mosaic), and the generated Rust code into a single, highly readable format, enhancing documentation and educational capabilities.

## Decision

We formally recognize these two modules as part of the "Nova" Developer Experience toolset.

- `alchemist.rs` is responsible for Python transpilation.
- `weave.rs` is responsible for generating educational Markdown artifacts.

Both components receive the "Analyzed Program" from the semantic analysis pipeline, mapping Greek syntax to output code.

## Consequences

- **Documentation**: Our architectural diagrams and documentation will accurately represent all tools in the ecosystem.
- **Educational Value**: `weave.rs` greatly simplifies teaching the mapping of morphology to code.
- **Maintenance Trade-off**: Supporting multiple exporters (Rust and Python) adds maintenance overhead. Changes to the core language semantics must now be implemented in two separate code generators.
