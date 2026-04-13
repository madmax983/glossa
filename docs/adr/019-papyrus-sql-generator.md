# 019. Add Papyrus SQL Generator Tool

Date: 2024-05-20

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler provides tools for analyzing, executing, and translating Ancient Greek source code. Our existing "Developer Experience (Nova)" toolset includes tools like the Alchemist (Python export) and Weave (Markdown generation).
Recently, a need has arisen to bridge the gap between ΓΛΩΣΣΑ's semantic types and database persistence logic. Specifically, developers need a streamlined way to extract defined `TypeDefinition` constructs (structs) and automatically generate corresponding SQL `CREATE TABLE` schemas.

## Decision

We have added "Papyrus" (`Πάπυρος`) to the "Developer Experience (Nova)" toolset (`src/tools/papyrus.rs`). Papyrus is an integrated tool that traverses the `AnalyzedProgram` from the Semantic Analyzer, finds `AnalyzedStatement::TypeDefinition` statements, and generates valid SQL `CREATE TABLE` schemas with appropriate column types mapped from `GlossaType`. It outputs these schemas using a beautifully formatted `comfy-table` interface.

## Consequences

* **Positive:** Developers can seamlessly generate SQL schemas directly from their Ancient Greek structs, improving developer experience and database integration.
* **Positive:** Papyrus adheres to the established architectural pipeline by acting as a consumer of the `AnalyzedProgram`, similar to the `Alchemist` and `Weave` tools.
* **Negative:** The tool introduces an additional maintenance burden, as it must be updated whenever `GlossaType` variants or structural definitions change in the semantic model.
