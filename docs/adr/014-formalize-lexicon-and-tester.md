# 14. Formalize Lexicon and Test Runner Tools

Date: 2025-02-24
Status: Accepted

## Context

As the ΓΛΩΣΣΑ compiler evolved, we added several tools to `src/tools/` to improve the developer experience (DX). While major tools like "The Nova Experience" (Cartographer, Mosaic, Mentor) were documented in ADR 013, two significant additions were made without formal architectural records:

1.  **The Lexicon (`src/tools/dictionary.rs`)**: A tool for looking up words in the built-in dictionary and performing morphological analysis.
2.  **The Judge (`src/tools/tester.rs`)**: A test runner that compiles Glossa files with `rustc --test`, enabling native unit testing.
3.  **The Stage (`src/tools/ui.rs`)**: A shared utility module for consistent terminal UI (spinners, emojis, colors).

These tools are crucial for the language's usability but their design motivations and trade-offs were left implicit. This lack of documentation creates "architectural debt" and obscures the system's boundaries.

## Decision

We formally recognize these modules as core components of the compiler toolset:

### 1. The Lexicon (Dictionary)
We treat `src/tools/dictionary.rs` as the authoritative interface for querying the `morphology` crate. It provides:
- Direct lookup in the static lexicon.
- Morphological analysis for unknown words.
- A user-friendly CLI output format.

### 2. The Judge (Tester)
We establish `src/tools/tester.rs` as the standard way to run tests in Glossa. It bridges the gap between the Glossa AST and Rust's testing infrastructure (`cargo test`).
- It parses and analyzes test files.
- It generates a temporary Rust file with `#[test]` attributes.
- It compiles and runs the binary, parsing the output to present results in a "Greek-first" format.

### 3. The Stage (UI)
We centralize all terminal output logic in `src/tools/ui.rs`. This ensures consistent styling (colors, symbols) across all tools and prevents duplication of "spinner" or "progress bar" logic.

## Consequences

### Positive
- **Clarity**: The role of every file in `src/tools/` is now documented.
- **Maintainability**: Future changes to testing or dictionary logic have a clear home.
- **Consistency**: The architecture diagram will now accurately reflect the dependencies (e.g., `Dictionary` depending on `Morphology`).

### Negative
- **Maintenance**: We must ensure these tools are kept up-to-date with changes in the core compiler (e.g., if `morphology` changes, `dictionary` must be updated).

## Compliance
This ADR satisfies the Codex requirement to document all structural components.
