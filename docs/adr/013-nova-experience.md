# 13. The Nova Experience

Date: 2025-02-23
Status: Accepted

## Context

The ΓΛΩΣΣΑ compiler introduces several novel concepts that can be challenging for new developers:
1.  **Ancient Greek Grammar**: Using case endings (Nominative, Accusative, Dative) instead of word order.
2.  **Architectural Complexity**: Understanding how types and traits interact in large codebases.
3.  **Unique Syntax**: Features like "The Perfect Participle" (memoized closures) and "The Optative Mood" (Optional types).

Without specialized tooling, the learning curve is steep, and debugging semantic misunderstandings (e.g., "Why is my subject being parsed as an object?") is difficult.

## Decision

We have implemented a suite of Developer Experience (DX) tools under the codename **Nova**. These tools are integrated directly into the compiler binary but guarded by the `nova` feature flag to manage binary size.

The Nova suite includes:

*   **The Mentor (ὁ Μέντωρ)**: An interactive CLI tutorial that guides users through the language basics with live verification.
*   **The Cartographer**: A visualization engine that generates Mermaid.js Class Diagrams from the analyzed program structure, revealing the relationships between Types and Traits.
*   **The Mosaic**: A semantic assembly visualizer that deconstructs statements into their grammatical constituents (Subject, Verb, Object, Indirect Object), proving the "free word order" capabilities.

## Consequences

*   **Improved Learnability**: Users can interactively learn the language and visualize how the compiler "thinks".
*   **Architectural Transparency**: Developers can generate maps of their code structure automatically.
*   **Debugging**: The Mosaic tool provides immediate feedback on how the Assembler is interpreting complex sentences.
*   **Binary Size**: Including these tools increases the compiler binary size. Therefore, they are opt-in via the `--features nova` flag.
