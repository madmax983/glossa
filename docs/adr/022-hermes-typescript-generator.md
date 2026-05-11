# 22. Hermes TypeScript Generator

Date: 2025-02-24
Status: Accepted

## Context

As the ΓΛΩΣΣΑ ecosystem expands, the ability to integrate with frontend applications becomes increasingly important. While tools like `Papyrus` (SQL) and `Alchemist` (Python) provide robust backend integration pathways, there is currently no direct mechanism to bridge Glossa structs (`εἶδος`) with modern frontend web applications, which predominantly use TypeScript.

Developers building full-stack applications with a Glossa backend need a way to ensure type safety across the network boundary without manually maintaining duplicate type definitions in TypeScript.

## Decision

We will implement "The Hermes" (`src/tools/hermes.rs`), a new developer tool following the "Exporter" pattern.

Hermes will transpile Glossa's semantically analyzed AST (`AnalyzedProgram`) directly into TypeScript `interface` declarations. It will map core Glossa types (e.g., `ἀριθμοῦ`, `ὀνόματος`, `σύνολον`) to their TypeScript equivalents (`number`, `string`, `Set`).

This tool will be gated behind the `nova` feature flag to maintain the core compiler's focus while enabling experimental developer experience enhancements.

## Consequences

### Positive
- **Full-Stack Integration:** Enables seamless type-sharing between Glossa backends and TypeScript frontends.
- **Eliminates Duplication:** Prevents the "two sources of truth" problem for data models.
- **Extends the Exporter Pattern:** Validates the robustness of the semantic AST for another external target.

### Negative
- **Maintenance Surface:** Mappings between Glossa types and TypeScript types will need to be maintained as new types are added to the language.
- **Complex Types:** Certain Glossa constructs (like full Result/Option monads) may not map perfectly 1:1 to idiomatic TypeScript without generating additional helper libraries.
