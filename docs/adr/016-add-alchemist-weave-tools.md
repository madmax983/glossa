# 16. Add Alchemist and Weave Tools

Date: 2026-03-15
Status: Accepted

## Context

The ΓΛΩΣΣΑ compiler ecosystem is expanding to include new output formats beyond the primary Rust backend.
While the primary backend targets Rust, Python is an excellent, dynamic target that aligns with the "scripting" feel of small ΓΛΩΣΣΑ programs. Additionally, there is a need for a structured way to export the codebase into a readable Markdown format, serving as a 'Rosetta Stone' to make it easy to see how Greek syntax maps to semantic meaning and compiled Rust code.

## Decision

We have added two new tools to the Developer Experience (Nova) boundary:
1.  **Alchemist (`src/tools/alchemist.rs`)**: An experimental transpiler that converts ΓΛΩΣΣΑ programs into Python scripts.
2.  **Weave (`src/tools/weave.rs`)**: An exporter that generates a 'Rosetta Stone' Markdown document combining ΓΛΩΣΣΑ source code, semantic assembly logic, and generated Rust code.

## Consequences

### Positive
- **Architectural Proof**: The Alchemist transpiler proves the independence of the semantic analysis phase from the Rust code generation phase, validating the modularity of the compiler pipeline.
- **Improved Documentation and Education**: The Weave tool provides an invaluable resource for users to understand the compilation process and the mapping between Greek syntax, semantic HIR, and the final Rust output.
- **Alternative Export Format**: Provides a dynamic scripting target (Python) for small programs.

### Negative
- **Maintenance Overhead**: Introduces the need to maintain additional exporter logic and a new Python backend alongside the primary Rust backend.
