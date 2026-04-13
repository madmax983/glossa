# 19. Papyrus SQL Generator

Date: 2026-04-11

## Status

Accepted

## Context

As the ΓΛΩΣΣΑ (GLOSSA) language ecosystem expanded to include full applications rather than just computational scripts, there was a growing need to integrate the type system with external persistent storage. Developers were manually translating Ancient Greek `εἶδος` (struct/type) definitions into database tables, a process that was both error-prone and tedious. We needed an automated way to bridge the semantic model of Glossa programs directly to SQL infrastructure.

## Decision

We introduced a new tool called `Papyrus` (`src/tools/papyrus.rs`), which acts as an SQL schema generator.

Papyrus hooks into the standard semantic analysis pipeline, taking the `AnalyzedProgram` as input. It scans for `AnalyzedStatement::TypeDefinition` nodes and generates corresponding `CREATE TABLE` SQL statements. It includes a custom mapping (`glossa_type_to_sql`) that translates Glossa types (like `Number`, `String`, `Boolean`, and `Option`) into SQL types (`BIGINT`, `TEXT`, `BOOLEAN`, and `JSONB` for complex collections). The tool also leverages `comfy-table` to present the generated schema in a structured, visually appealing terminal UI, maintaining consistency with other Nova tools like `Alchemist` and `Mosaic`.

## Consequences

*   **Positive:** Developers can now seamlessly generate database schemas directly from their Ancient Greek type definitions, reducing the friction of setting up persistent data stores for Glossa applications.
*   **Positive:** The type translation logic centralizes the mapping rules between Glossa and standard SQL, preventing ad-hoc conversion scripts.
*   **Negative:** The SQL generation is currently quite basic; it relies heavily on PostgreSQL-specific idioms (like `JSONB` for collections) and does not yet handle complex relational constraints (like foreign keys) or dialects other than standard SQL/Postgres.
