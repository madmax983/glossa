# 17. Add Weave Tool

Date: 2026-03-14
Status: Proposed

## Context

As the project scales, users and learners need a way to export their codebase into a structured, readable Markdown format for documentation and educational purposes, making it easier to see how Greek syntax maps to semantic meaning and compiled Rust code.

## Decision

We have implemented the "Weave" functionality (`src/tools/weave.rs`), an exporter that generates a 'Rosetta Stone' Markdown document combining Glossa source code, semantic assembly logic, and generated Rust code.

## Consequences

### Positive
- **Educational Value**: Greatly enhances the educational aspect by creating a direct, visual mapping between the source, its assembly, and its final code.
- **Documentation Aid**: Allows users to quickly generate documentation demonstrating how the language compiles.

### Negative
- **Dependency on Mosaic and Codegen**: Tightly couples this tool to the internal structure of both the Mosaic assembly visualization tool and the Rust codegen system.
