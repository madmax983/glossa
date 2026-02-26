# 14. Formalize Compiler Tool Persona & Roles

Date: 2024-10-25

## Status

Accepted

## Context

As the ΓΛΩΣΣΑ compiler ecosystem grows, the number of auxiliary tools and utilities has increased. Initially, these were just loose collections of functions (e.g., `dictionary.rs`, `tester.rs`). However, as they have evolved into full-fledged components of the "Nova" developer experience, referring to them by their filenames or generic names ("the dictionary tool") has become dry and fails to convey their specific role in the user's journey.

We want the compiler to feel like a coherent world, not just a CLI tool. To achieve this, we need strong metaphors that align with the "Ancient Greek" theme of the language.

## Decision

We have decided to formally designate specific personas for the core compiler tools. These names will be used in documentation, CLI output, and architectural diagrams.

| Module | Old Name | New Persona | Role | Symbol |
|--------|----------|-------------|------|--------|
| `src/tools/dictionary.rs` | Dictionary | **The Lexicon** | The Source of Truth for Words. Handles lookups and definitions. | 📚 |
| `src/tools/tester.rs` | Tester | **The Judge** | Verifies Correctness. Runs tests and passes judgment on code. | ⚖️ |
| `src/tools/ui.rs` | UI | **The Stage** | Presentation Layer. Handles how information is presented to the user (spinners, colors). | 🎭 |
| `src/tools/narrator.rs` | Narrator | **The Bard** | Storyteller. Translates the AST into a human-readable "Scroll of Logic". | 📜 |

## Consequences

*   **Documentation:** All future documentation (including `AGENTS.md` and `architecture.md`) will refer to these tools by their Persona names.
*   **CLI Output:** The tools themselves will adopt these personas in their output. For example, the Tester might say "The Judge has spoken" instead of just "Tests passed".
*   **Code Structure:** While the filenames will remain snake_case (`tester.rs`, `dictionary.rs`) for Rust conventions, the internal structs and public APIs may adopt the new nomenclature where appropriate.
*   **Mental Model:** Users will have a clearer mental model of the system:
    *   Consult **The Lexicon** for words.
    *   Ask **The Bard** to explain code.
    *   Face **The Judge** to verify correctness.
