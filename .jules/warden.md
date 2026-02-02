# Warden's Journal

## 2024-05-22 - Logic Bug in Morphological Stripping

**Threat:** Logic bug in `src/semantic/patterns.rs`. The use of `trim_end_matches("ου")` removes *all* trailing occurrences of "ου". For a variable like "λουλου" (hypothetical), it would strip it to "λ" instead of "λουλ". This can corrupt variable names and lead to undefined behavior in the semantic analysis (reference errors).

**Defense:** Switch to `strip_suffix("ου")` which removes only the last occurrence, preserving the stem if it naturally ends in the pattern.

**Severity:** Low (Logic bug/DoS via compilation error), but strictly incorrect string handling.

## 2026-02-02 - Code Generation Panic (DoS)

**Threat:** Unhandled Unicode characters in `src/codegen/rust.rs`'s `transliterate` function allowed arbitrary characters to be passed to `format_ident!`, causing compiler panics (DoS) when compiling code with specially crafted variable names (e.g. using Greek Koronis `᾽`).

**Defense:** Enforced strict ASCII allowlist in `transliterate`. Any character not matching `is_ascii_alphanumeric()` or `_` is replaced with `_`.

**Severity:** Medium (DoS via compiler crash).
