# 4. Slot-Based Semantic Assembly

Date: 2024-10-24
Status: Accepted

## Context

Standard compiler architectures (like those for C, Java, or Rust) typically use a recursive-descent parser or similar technique where the *position* of a token determines its semantic role. For example, in `a = b`, `a` is the target and `b` is the value because of their positions relative to `=`.

Ancient Greek, however, is a **free word order** language. The meaning of a sentence is determined by **inflection** (morphological endings), not word order.
- `ὁ ἄνθρωπος τὸν λόγον λέγει` (The man says the word) - SOV
- `λέγει τὸν λόγον ὁ ἄνθρωπος` (Says the word the man) - VSO
- `τὸν λόγον λέγει ὁ ἄνθρωπος` (The word says the man) - OVS

All of these are grammatically correct and mean the same thing (semantically). A standard AST walker that expects a fixed structure (e.g., `Statement -> Subject Verb Object`) cannot handle this efficiently without combinatorial explosion in the grammar.

## Decision

We have implemented a **Slot-Based Semantic Assembler** (`src/semantic/assembler.rs`).

Instead of building a rigid AST that dictates semantic roles, the parser produces a linear stream of tokens. These tokens are fed into the `Assembler`, which acts as a state machine with specific "Slots" for semantic roles:

- **Subject Slot**: Filled by words in the **Nominative** case.
- **Object Slot**: Filled by words in the **Accusative** case.
- **Indirect Object Slot**: Filled by words in the **Dative** case.
- **Verb Slot**: Filled by **Verbs**.
- **Modifiers**: Adjectives and Genitives are accumulated in lists.

The Assembler is agnostic to the order of arrival. `Feed(Subject) -> Feed(Verb)` results in the same internal state as `Feed(Verb) -> Feed(Subject)`.

Once a statement boundary (e.g., a period) is reached, the Assembler `finalize()`s the state into an `AssembledStatement`. This structure is then passed to the `Conversion` module to be interpreted as a specific high-level construct (Assignment, Function Call, etc.).

## Consequences

### Positive
- **Authenticity**: Enables true free word order, a core requirement for a "Greek-native" programming language.
- **Flexibility**: We can easily add new slots or modifiers without rewriting the core parsing logic.
- **Robustness**: Error recovery is often easier; if we see a second Subject, we can report a "Double Subject" error immediately, rather than failing a specific grammar rule.

### Negative
- **Two-Phase Semantics**: We effectively have two "ASTs": the raw parse tree and the `AssembledStatement`. This adds a conversion step.
- **Complexity**: The `Conversion` module (`src/semantic/conversion.rs`) and `Patterns` module (`src/semantic/patterns.rs`) must now handle the interpretation of these assembled slots. For example, distinguishing "Variable Declaration" from "Function Call" requires heuristic checks on the contents of the slots, rather than just matching a grammar rule.
