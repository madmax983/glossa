# 16. Alchemist and Weave Experimental DX Tools

Date: 2026-03-30
Status: Accepted

## Context

Two new tools have been added to the codebase for transpiling to Python and generating Rosetta Stone Markdown files. The Alchemist (`src/tools/alchemist.rs`) serves as an experimental Python exporter to prove the semantic independence from Rust codegen, while Weave (`src/tools/weave.rs`) acts as an exporter that generates a 'Rosetta Stone' Markdown document combining Glossa source code, semantic assembly logic, and generated Rust code.

## Decision

We formally recognize these modules under the `nova` feature flag in `src/tools/mod.rs` to aid with developer experience and education without polluting the core compiler pipeline. They are to be integrated tightly with the Semantic Analyzer (`src/semantic`), taking the `AnalyzedProgram` as input.

## Consequences

### Positive
- **Learning Aids**: Provides users with alternative export formats and comprehensive documentation tools.
- **Independence**: Proves that the semantic phase is truly independent from the primary Rust codegen backend.

### Negative
- **Maintenance Burden**: Adds maintenance overhead for transpiling to new target languages and keeping the Markdown outputs synced with structural changes.
