# Warden's Journal

## 2024-05-22 - Logic Bug in Morphological Stripping

**Threat:** Logic bug in `src/semantic/patterns.rs`. The use of `trim_end_matches("ου")` removes *all* trailing occurrences of "ου". For a variable like "λουλου" (hypothetical), it would strip it to "λ" instead of "λουλ". This can corrupt variable names and lead to undefined behavior in the semantic analysis (reference errors).

**Defense:** Switch to `strip_suffix("ου")` which removes only the last occurrence, preserving the stem if it naturally ends in the pattern.

**Severity:** Low (Logic bug/DoS via compilation error), but strictly incorrect string handling.

## 2026-02-02 - Code Generation Panic (DoS)

**Threat:** Unhandled Unicode characters in `src/codegen/rust.rs`'s `transliterate` function allowed arbitrary characters to be passed to `format_ident!`, causing compiler panics (DoS) when compiling code with specially crafted variable names (e.g. using Greek Koronis `᾽`).

**Defense:** Enforced strict ASCII allowlist in `transliterate`. Any character not matching `is_ascii_alphanumeric()` or `_` is replaced with `_`.

**Severity:** Medium (DoS via compiler crash).

## 2025-05-22 - Variable Collision & Index Wrapping

**Threat:**
1. **Identifier Collision:** `transliterate` mapped all invalid characters to `_`, causing variable shadowing/collision for distinct Greek characters (e.g. `ϟ` and `ϛ` both became `_`).
2. **Index Wrapping:** Generated Rust code cast `i64` indices to `usize` without checking for negativity. `-1` wrapped to `usize::MAX`, causing a panic.
3. **DoS Vectors:** Unbounded file reading and unbounded sentence assembly allowed memory exhaustion.

**Defense:**
1. **Unique Transliteration:** Map invalid characters to `_u{hex}_` to ensure uniqueness.
2. **Checked Indexing:** Injected runtime check `if idx < 0 { panic!(...) }` before indexing.
3. **Resource Limits:** Enforced `MAX_FILE_SIZE` (1MB) and `MAX_TOKENS` (1000/stmt).

**Severity:** Medium (DoS & Logic bugs).

## 2026-06-01 - Logic Bug in Morphology & REPL DoS

**Threat:**
1. Logic bug in `src/morphology/conjugation.rs`: `trim_end_matches('θ')` over-stripped stems ending in theta.
2. REPL DoS: Unbounded history growth in `ReplContext`.

**Defense:**
1. Switched to `strip_suffix('θ')` in conjugation.
2. Enforced `MAX_REPL_BINDINGS` (50) and `MAX_REPL_SOURCE_LEN` (50KB) in `src/main.rs`.

**Severity:** Low (Logic bug) / Medium (DoS).

## 2026-06-02 - Identifier Collision Logic Bug

**Threat:** Identifier collision in `src/codegen/rust.rs`. `sanitize_name` mapped single Greek letters (e.g., `σ`) to their full names (`sigma`), causing collisions with variables named with the full name (`σίγμα`). This allowed variable shadowing and potential logic errors. Also `ψ` collided with `πσ`.

**Defense:** Removed special single-letter mapping in `sanitize_name` and enforced strict transliteration in `transliterate`. `σ` now maps to `s`, while `σίγμα` maps to `sigma`. `ψ` maps to `_u3c8_` (hex encoded) to avoid collision with `ps`.

**Severity:** Medium (Logic Bug).

## 2026-06-03 - Keyword Injection & Identifier Collision

**Threat:** The compiler generated Rust identifiers directly from transliterated user input without preventing collisions with Rust keywords (e.g., `let`, `fn`, `struct`) or internal compiler variables (e.g., `idx` in index access). This allowed a malicious or accidental user to generate invalid Rust code (`let struct = 5;`) or buggy code (shadowing internal logic), causing Denial of Service (compilation failure) or logic errors.

**Defense:** Updated `sanitize_name` in `src/codegen/utils.rs` to prefix all user-defined identifiers with `g_` (e.g., `g_struct`, `g_let`). This namespaces user identifiers away from Rust keywords and generated internal variables. Standard library methods (e.g., `push`, `len`) are whitelisted to avoid prefixing.

**Severity:** Medium (DoS via compilation failure & Logic errors).
