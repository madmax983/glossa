# 020. Add Catalog Tool to Developer Experience Tools

Date: 2026-04-19

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler includes the "Lexicon" module (`src/morphology/lexicon.rs`) which acts as the built-in dictionary containing words and their parts of speech, lemma forms, meanings, and rust equivalents.
However, exploring this lexicon visually from the command line was difficult. To improve the developer experience and provide an easy way to view the available vocabulary categorized by part of speech, we added the "Catalog" (`ὁ Κατάλογος`) tool in `src/tools/catalog.rs`.

## Decision

We have added the "Catalog" tool to the "Developer Experience (Nova)" toolset (`src/tools/catalog.rs`).
The Catalog provides a visual, formatted output using `comfy-table` to explore the static lexicon entries. It groups words by their Part of Speech and prints them to the terminal.

## Consequences

*   **Positive:** Developers and users have an easy, structured way to explore the internal vocabulary of ΓΛΩΣΣΑ via the CLI.
*   **Positive:** Follows the established pattern of implementing discrete developer tools within the `src/tools/` directory.
*   **Negative:** Adds a new tool that must be maintained as part of the `glossa` toolset.
