# 024. Add Scholar (ὁ Σχολαστικός) API Doc Generator

Date: 2026-08-19
Status: Proposed

## Context
To ensure ΓΛΩΣΣΑ code can be documented effectively without forcing users to read raw source files, an automated documentation tool is needed. The "Scholar" tool (`src/tools/scholar.rs`) was introduced to bridge the gap between the semantic AST and human-readable references by extracting structures (`εἴδη`), traits (`χαρακτῆρες`), and functions (`ἔργα`) into a clean, GitHub-flavored Markdown file (`doc.md`). This new component must be documented to maintain architectural transparency.

## Decision
We formally acknowledge the **Scholar** tool as part of the "Developer Experience (Nova)" container (`src/tools/`).
Scholar is responsible for generating Markdown API documentation by consuming the `AnalyzedProgram` from the Semantic Analyzer.

## Consequences
- **Positive:** Aligns with documentation-as-code principles. Provides a standardized way to generate human-readable API references.
- **Negative:** The tool must continuously adapt to new language features (like new types of declarations) to ensure the generated documentation remains complete.
