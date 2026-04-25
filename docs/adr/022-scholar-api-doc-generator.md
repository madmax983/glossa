# 022. The Scholar (ὁ Σχολαστικός) - API Doc Generator

Date: 2026-04-26

## Status

Proposed

## Context

We needed a way to generate API documentation from Glossa code, similar to `rustdoc`, but tailored to Glossa's linguistic structures. Users need to be able to understand the types, traits, and functions exposed by a program without having to read the source code manually.

## Decision

We implemented `src/tools/scholar.rs` as a new tool to parse the semantic model and output Markdown files documenting the APIs of the provided `.γλ` file. It iterates over the types, traits, and functions in the program's scope and generates a structured `.doc.md` file.

## Consequences

- Provides native documentation capabilities, improving the developer experience.
- Developers can easily share API references for their Glossa code.
- Requires maintaining another tool within the Nova boundary, increasing the complexity of the compiler's developer tools suite.
