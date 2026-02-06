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

## 2026-06-03 - Nested Expression Truncation

**Threat:** Logic bug in `src/semantic/expressions.rs` and `src/semantic/conversion.rs`.
1. `analyze_argument_expr` silently truncated nested phrases (e.g., `(1 2 sum)`) to their first term (`1`), ignoring subsequent terms and operators. This could allow logic errors or assertion bypasses.
2. `extract_value` ignored nested phrases entirely, causing bindings like `x = (1 + 2)` to default to `0` (silent data loss).

**Defense:**
1. Updated `analyze_argument_expr` to use a fresh `Assembler` to correctly analyze the full nested phrase, preserving operator order and terms.
2. Updated `extract_value` to correctly extract and analyze nested phrases.
3. Used `scope.child()` for nested analysis to prevent variable leakage.

**Severity:** High (Logic Bug / Silent Data Loss).
