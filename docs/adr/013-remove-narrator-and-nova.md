# 13. Remove Narrator and Nova Tools

Date: 2025-02-23
Status: Accepted

## Context

The ΓΛΩΣΣΑ compiler project included several experimental tools designed to enhance the learning experience:
- **Narrator** (formerly `Bard`): A tool to translate Greek code into an English narrative (e.g., "The man says the word").
- **Nova Suite**: A set of interactive tools including:
    - **Mentor**: An interactive tutorial system.
    - **Mosaic**: A visualization tool for the semantic assembler.
    - **Cartographer**: A planned map-like visualization.

While these tools were conceptually interesting, they introduced significant maintenance burden. The `Narrator` required keeping a separate "English generation" logic in sync with the core semantic analysis, effectively requiring every language feature to be implemented twice. The `Nova` tools relied on complex terminal UI interactions that were often fragile and distracted from the core mission of building a robust compiler.

Furthermore, the "Razor" philosophy (mentioned in ADR 010) mandates that we should not accumulate experimental or tentative code. The primary focus of ΓΛΩΣΣΑ is to be a working compiler where Ancient Greek morphology encodes programming semantics, not necessarily an educational platform.

## Decision

We have decided to **remove** the `Narrator` and the entire `Nova` suite from the codebase.

1.  **Remove** `src/tools/narrator.rs`.
2.  **Remove** `src/tools/mentor.rs`.
3.  **Remove** `src/tools/mosaic.rs`.
4.  **Remove** the `nova` feature flag from `Cargo.toml`.
5.  **Remove** associated CLI commands (`bard`, `mentor`, `mosaic`) and logic.
6.  **Delete** associated tests (`tests/narrator_coverage.rs`).
7.  **Preserve** useful tests:
    -   `tests/nova_coverage.rs` was identified as containing tests for the `tester` tool (the "Judge"), not the `nova` feature. It has been renamed to `tests/tester_tests.rs`.
    -   `tests/nova_numerals.rs` tested core numeral parsing logic. It has been renamed to `tests/numerals_tests.rs`.

## Consequences

### Positive
-   **Reduced Complexity**: The codebase is significantly smaller and easier to maintain.
-   **Focus**: Development effort can now be fully directed towards the core compiler pipeline (parsing, semantics, codegen).
-   **Build Times**: Removing these modules and their dependencies (like the complex logic in `narrator`) may slightly improve build times.
-   **Clarity**: Users are presented with a standard compiler interface (`run`, `build`, `check`) rather than a mix of compiler and educational tools.

### Negative
-   **Loss of Feature**: The unique "code-to-story" translation feature is gone. This was a differentiator for the project but was deemed non-essential.
-   **Loss of Visualization**: The `Mosaic` tool provided insight into the assembler's internal state. Developers debugging the assembler will now rely on standard logging and debug prints.

### Mitigation
-   The core compiler still supports `check` and `highlight`, which provide essential feedback.
-   The preserved tests ensure that core functionality (`tester`, `numerals`) remains verifying correctly.
