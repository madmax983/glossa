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

## 2026-06-03 - Rust Keyword Collision & Namespace Pollution

**Threat:**
1. **Keyword Collision:** User identifiers like `if` or `fn` (valid in Glossa/ASCII) transliterated directly to Rust keywords, causing compilation errors.
2. **Namespace Pollution:** User methods named `len` or `push` could shadow standard library methods, breaking generated code for standard types like `Vec`.

**Defense:**
1. **Namespace Isolation:** Modified `sanitize_name` to prefix ALL user-defined identifiers with `g_`.
2. **Std Lib Preservation:** Implemented `is_std_method` and `is_std_type` allowlists in code generation to detect calls to standard library methods and preserve their original names (e.g. `len`) instead of prefixing them.

**Severity:** Medium (Logic Bug / Compilation Failure).

## 2026-06-04 - HashDoS in Resolver & Iterator Prefixing Bug

**Threat:**
1. **HashDoS in Resolver:** User-controlled variable/type names were stored in `FxHashMap` (from `rustc-hash`), which uses a non-cryptographic hash function. A malicious source file with many colliding identifiers could cause quadratic lookup times (DoS) during semantic analysis.
2. **Broken Code Generation:** Iterator methods (e.g., `map`, `filter`) on intermediate `Unknown` types were incorrectly sanitized (prefixed with `g_`), resulting in invalid Rust code (e.g., `.g_map()`) that failed to compile.

**Defense:**
1. **Secure Hashing:** Replaced `FxHashMap` with `std::collections::HashMap` (SipHash) in `src/semantic/resolver.rs` for all user-controlled scopes.
2. **Standard Method Allowlist:** Added `GlossaType::Unknown` to `is_std_type` in `src/codegen/rust.rs` to prevent prefixing standard methods on inferred/iterator types.

**Severity:** Medium (DoS via CPU exhaustion & Compilation Failure).

## 2026-06-05 - Stack Overflow DoS in Semantic Analysis

**Threat:** Stack Overflow (DoS) in `src/semantic/mod.rs` and `src/codegen/rust.rs`. Deeply nested expression trees (e.g., `1 + 1 + ...`) caused unbounded recursion during semantic analysis and code generation, crashing the compiler with a stack overflow.

**Defense:** Implemented `check_program_depth` in `src/semantic/mod.rs` to enforce a strict recursion limit (`MAX_EXPRESSION_DEPTH = 200`). Analysis now fails gracefully with `GlossaError::LimitExceeded` if the depth is exceeded.

**Severity:** High (DoS via compiler crash).

## 2026-06-06 - Insecure Cache Key Hashing

**Threat:** The build cache in `src/tools/cache.rs` used `std::collections::hash_map::DefaultHasher` to compute cache keys from file paths. `DefaultHasher` is not cryptographically secure and its stability across releases or environments is not guaranteed. This could lead to cache collisions (incorrect binary execution) or unstable keys (cache thrashing/DoS).

**Defense:** Replaced `DefaultHasher` with `sha2::Sha256` to ensure deterministic, cryptographically secure, and collision-resistant cache keys (64-character hex strings).

**Severity:** Medium (Cache Collision / DoS).
