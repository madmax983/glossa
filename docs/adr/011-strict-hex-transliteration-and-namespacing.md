# 11. Strict Hex Transliteration and Namespacing

Date: 2025-05-23

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler generates Rust code from Ancient Greek source. This process requires mapping Greek identifiers (e.g., `χρήστης`, `ἀριθμός`) to valid ASCII Rust identifiers.

Initially, a "friendly" transliteration strategy was considered, mapping characters to their phonetic equivalents (e.g., `χ` &rarr; `ch`, `φ` &rarr; `ph`, `θ` &rarr; `th`). However, this approach introduced critical collision risks:

1.  **Digraph Collisions**: If `χ` maps to `ch`, a user variable named `χ` collides with a user variable named `ch` (if Latin script is used or mixed).
2.  **Keyword Collisions**: A Greek word might transliterate to a Rust keyword (e.g., `εἰ` &rarr; `if` purely by accident or similarity, or if a user defines a variable named `if` in a Latin-script block).
3.  **Ambiguity**: Polytonic Greek has many accented characters. Stripping accents (normalization) solves some issues but creates ambiguity between distinct words if not handled carefully.
4.  **Security**: Malicious input could theoretically construct identifiers that clash with internal compiler variables or keywords.

## Decision

We have adopted a **Strict Hex Encoding** strategy combined with **Namespacing** for all user-defined identifiers.

### 1. The `g_` Namespace Prefix
All user-defined identifiers in the generated Rust code are prefixed with `g_`.
- `χρήστης` &rarr; `g_...`
- `if` (if used as a variable) &rarr; `g_if`

This ensures that no user-defined variable can ever collide with a Rust keyword (`if`, `fn`, `match`, etc.) or standard library types unless explicitly intended (and even then, the prefix protects it).

### 2. Hex Encoding for Ambiguous Characters
We only map Greek characters to Latin characters when the mapping is **1:1 and unambiguous**.
- `α` &rarr; `a`
- `β` &rarr; `b`
- `ξ` &rarr; `x` (Maps to a single character, safe)

For characters that typically map to digraphs (`th`, `ph`, `ch`, `ps`) or have no direct equivalent, we use **Unicode Hex Encoding** (`_uXXXX_`).
- `θ` (theta) &rarr; `_u3b8_` (Not `th`)
- `φ` (phi) &rarr; `_u3c6_` (Not `ph`)
- `χ` (chi) &rarr; `_u3c7_` (Not `ch`)
- `ψ` (psi) &rarr; `_u3c8_` (Not `ps`)
- `ϟ` (koppa) &rarr; `_u3df_`

This ensures that `χ` (chi) never collides with the sequence `c` + `h`.

## Consequences

### Positive
- **Guaranteed Safety**: It is mathematically impossible for a valid Greek identifier to collide with a Rust keyword or another distinct Greek identifier (assuming normalization is applied first).
- **Simple Validation**: The rules are mechanically simple and robust.
- **No Reserved Words**: Users can theoretically use any Greek word as a variable name without fear of hitting a reserved keyword in the target language (Rust).

### Negative
- **Readability**: The generated Rust code is significantly less readable. `χρήστης` becomes `g__u3c7_rhsths` instead of `g_christos`.
- **Debugging**: Inspecting the generated code requires mental translation or a decoder tool.

### Mitigation
Since the generated Rust code is an intermediate artifact intended for the Rust compiler (not for human maintenance), the readability cost is considered acceptable in exchange for absolute compilation safety.
