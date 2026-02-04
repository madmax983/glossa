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
