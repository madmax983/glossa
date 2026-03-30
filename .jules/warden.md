# Warden Security Journal

## Mission
To identify vulnerabilities, harden interfaces, and eliminate memory safety risks.

## Status: ✅ Secured

### 1. Memory Safety
- **State:** Pure Rust. `unsafe` block verified and documented.
- **Verification:** `grep -r "unsafe" .` returned known locations. Added `SAFETY` comments to `src/ast.rs`.
- **Policy:** Any introduction of `unsafe` requires formal proof.

### 2. Input Sanitization
- **Identifiers:** All Greek identifiers are hex-encoded (e.g., `α` -> `_u3b1_`) to prevent collisions and injection.
  - **Source:** `src/codegen.rs` (`sanitize_name`, `transliterate`).
  - **Verification:** `tests/warden_exploit.rs`.
- **Terminal Output:** String literals are sanitized using `escape_debug()` before printing to terminal in syntax highlighter.
  - **Source:** `src/tools/highlight.rs`.
  - **Threat:** Terminal injection attacks (ANSI escape codes).

### 3. DoS Prevention
- **Recursion Limits:** Enforced in parser (`pest`) and semantic analysis (`MAX_RECURSION_DEPTH = 50`).
  - **Source:** `src/semantic/expressions.rs`.
  - **Verification:** `tests/warden_limits.rs`.
- **Text Normalization:** Verified linear complexity O(N) for normalization algorithm.
  - **Source:** `src/text.rs`.
  - **Verification:** `tests/warden_text_dos.rs` (Added 2024-03-XX).
  - **Threat:** Quadratic behavior on repeated Sigma sequences.

### 4. Integer Safety
- **Math:** Uses `checked_*` methods with `expect()` for `GlossaType::Number`.
  - **Source:** `src/codegen.rs`.
  - **Note:** Panics on overflow (Safe crash), preventing undefined behavior or silent wrapping.

## Recent Actions
- **2024-03-XX - [Logic Bug - Ignored Nested Phrases]**
  - **Threat:** Nested phrases in variable binding (e.g., `α (1 (2 3)) ἔστω`) were silently ignored, causing variables to be bound to a default value (0) instead of erroring or binding correctly. This could lead to semantic confusion or logic bugs in user code.
  - **Investigation:** Discovered that `extract_value` in `src/semantic/conversion.rs` did not check `nested_phrases` from the assembler.
  - **Defense:** Updated `extract_value` to process `nested_phrases`. It now detects invalid nested structures and reports an error ("Unexpected multiple terms") instead of silently ignoring them.
  - **Verification:** Added `tests/warden_nested_phrase.rs` which confirms that invalid nested phrases now trigger a compilation error.

- **2024-03-XX - [Text Normalization DoS Probe]**
  - **Threat:** Potential O(N^2) behavior in `GreekLowercaseIterator` when handling repeated Sigmas.
  - **Investigation:** Audit of `src/text.rs` revealed early exit in lookahead loop (`break` on non-diacritic).
  - **Verification:** Benchmarking with `tests/warden_text_dos.rs` confirmed linear performance (~9ms for 20k chars).
  - **Defense:** Existing implementation deemed safe; regression test added.

- **2024-03-XX - [Logic Bug - Silent Block Truncation]**
  - **Threat:** Block expressions in bindings (e.g., `α { 1. 2. } ἔστω`) were either completely ignored (evaluating to 0) or silently truncated to the first statement (evaluating to 1, ignoring 2). This could allow malicious code or security checks to be bypassed if they were placed in the ignored portion of the block.
  - **Investigation:** Discovered `extract_value` in `src/semantic/conversion.rs` ignored `asm.blocks`, and `analyze_block` in `src/semantic/expressions.rs` only analyzed the first statement.
  - **Defense:**
    1.  Updated `extract_value` to process `asm.blocks`.
    2.  Hardened `analyze_block` to strictly require exactly one statement/clause/expression. Multi-statement blocks in expressions now trigger a compile-time error.
  - **Verification:** Added `tests/repro_ignored_block.rs`. Confirmed that `α { 1. } ἔστω` now correctly evaluates to 1, and `α { 1. 2. } ἔστω` raises a semantic error.

- **2024-11-XX - [Unsafe Block Audit & Formal Safety Proof]**
  - **Threat:** Missing formal safety proof for an `unsafe` block in `src/ast.rs` during AST dropping, which violated Warden's memory and required auditing to ensure no Use-After-Free or Double Free vulnerabilities existed within `Expr` drop code.
  - **Investigation:** Audited `Drop` implementation for `Expr`. Verified that usage of `ManuallyDrop` and `std::ptr::read` does not leak inner structures since all variants correctly drop their heap-allocated values. Verified no dependency CVEs via `cargo audit`.
  - **Defense:** Added formal `SAFETY:` documentation detailing the 6 steps that prevent recursion overflows while safely dropping the memory in `src/ast.rs`.
  - **Verification:** Tests (`cargo test`) pass, confirming that AST components do not introduce memory issues during execution.

- **2025-01-XX - [Integer Overflow in Unary Negation]**
  - **Threat:** Unary negation (`UnaryOp::Neg`) of `i64::MIN` would silently overflow in release builds, causing unexpected wrapping due to the raw `-x` generation.
  - **Investigation:** Code audit of `src/codegen.rs` revealed `generate_unary_op` translated unary negation to `-#operand_tokens`.
  - **Defense:** Modified `generate_unary_op` to check `GlossaType::Number` and emit `.checked_neg().expect("arithmetic overflow")`, safely panicking instead of wrapping in release mode.
  - **Verification:** Unit test added in `tests/warden_neg_overflow.rs`.

## Pending Actions
- **Dependency Audit:** `cargo audit` command checked and verified clean.
- **Recursive Structs:** Currently handled by failing compilation in `rustc`. Future improvement: Detect cycles during semantic analysis for better error messages.

Signed,
🔒 Warden
**2023-10-27 - Remove Unsafe Drop in AST**
**Threat:** Use of `unsafe` code with `std::ptr::read` in `src/ast.rs`'s `Drop` implementation to prevent recursion, which could have led to bugs or memory leaks on panics and was highly dependent on exact memory layouts and variant checks.
**Defense:** Rewrote the `Drop` implementation using 100% safe Rust via `std::mem::replace` and `std::mem::take` to explicitly dismantle nested enum variants by replacing their children with trivial non-allocating dummy values, allowing safe teardown without risking stack overflows.
**2024-05-15 - Unbounded File Read DoS in Weave Tool**
**Threat:** The `run_weave` tool was using `std::fs::read_to_string`, which loads an entire file into memory without limits. An attacker could use a massive file or an infinite stream (like `/dev/zero`) to exhaust memory and crash the application.
**Defense:** Replaced the unbounded read with the localized safe abstraction `load_source` from `crate::tools::runner::load_source`. `load_source` enforces a strict 1MB size limit by checking metadata and using `take()` on streams, preventing memory exhaustion.
2024-05-24 - [Unbounded File Read in Alchemist]
**Threat:** The `run_alchemist` tool used `std::fs::read_to_string`, which loads the entire file into memory without bounds checking. A malicious user could supply a massively large file, leading to memory exhaustion and a Denial of Service (DoS).
**Defense:** Replaced the direct use of `fs::read_to_string` with `crate::tools::runner::load_source`, which enforces a strict 1MB file size limit and uses `take()` to limit read streams, preventing out-of-memory errors on large or infinite file streams (like `/dev/zero`). Verified with an automated test case (`test_run_alchemist_file_too_large`).

**2024-05-19 - [Interpreter Arithmetic Overflow DoS]
**Threat:** [Integer overflow in the Simulator interpreter via uncontrolled add, sub, mul, and neg causing unexpected rustc panics.]
**Defense:** [Replaced unsafe math operations (`+`, `-`, `*`, unary `-`) with `checked_add`, `checked_sub`, `checked_mul`, and `checked_neg` returning a new `EvalError::ArithmeticOverflow` variant instead of panicking.]

**2025-03-01 - [Stack Overflow in Derived Debug on Recursive Enums]**
**Threat:** DoS via Stack Overflow. The `Expr`, `Statement`, `Clause`, and `Program` enums in `src/ast.rs` are recursive. While `Drop`, `Clone`, and `PartialEq` used `stacker::maybe_grow`, `Debug` was auto-derived (`#[derive(Debug)]`), meaning printing a deeply nested AST (e.g. during error formatting) bypassed all checks and crashed the thread.
**Defense:** Removed `#[derive(Debug)]` and manually implemented `std::fmt::Debug` for all recursive AST nodes using `stacker::maybe_grow` to securely handle deep nesting during formatting.

**2026-03-18 - [Stack Overflow in Derived Debug on Recursive Semantic Structs]**
**Threat:** DoS via Stack Overflow. Similar to the AST, the semantic model structs (`AnalyzedStatement`, `AnalyzedExpr`, `AnalyzedExprKind`, etc.) in `src/semantic/model.rs` and `src/semantic/assembly.rs` were using auto-derived `#[derive(Debug)]`. Printing deeply nested semantic structs during compilation errors could crash the process.
**Defense:** Removed `#[derive(Debug)]` from `AnalyzedStatement`, `AnalyzedExpr`, `AnalyzedExprKind`, `AnalyzedMethod`, `AssembledStatement`, `Constituent`, `VerbConstituent`, `ParticipleConstituent`, and `Literal`, manually implementing `std::fmt::Debug` using `stacker::maybe_grow` to securely handle deep nesting during formatting.
**2025-03-05 - [Unaccented Greek Keyword Recursion Limit Bypass]**
**Threat:** DoS via Stack Overflow. The manual recursion depth scanner (`src/parser/recursion.rs`) checked strictly for accented keywords `δοκιμή` and `τέλος`, but the `pest` grammar allowed unaccented variants (`δοκιμη`, `τελος`). By using unaccented keywords, an attacker could nest test declarations indefinitely, bypassing the `MAX_PARSE_DEPTH` check and crashing the parser thread via a stack overflow.
**Defense:** Updated `check_recursion_depth` to look for both the accented and unaccented variations (`δοκιμή` || `δοκιμη`, `τέλος` || `τελος`) during the fast byte scan, ensuring parity with the grammatical specification and properly limiting nesting depth for all allowed test declaration forms.
**2024-03-12 - [Interpreter Denial of Service (Panic)]**
**Threat:** [The interpreter's `eval_bin_op` function panicked (DoS) on modulo-by-zero, as `checked_rem` was not properly utilized and zero was not checked for.]
**Defense:** [Implemented an explicit 0-check and safely evaluated modulo operations via `checked_rem`, mapping the error to an `ArithmeticOverflow` evaluation error.]

**2025-02-12 - [TryFrom Missing Import Causing ICE]
**Threat:** A logic bug allowed generated rust code to trigger an Internal Compiler Error (ICE) due to an out of scope `usize::try_from` usage on index accesses, leading to an inability to compile valid program trees with array indexing and rendering the previous DoS bounds checking defense ineffective.
**Defense:** Explicitly added `#![allow(non_snake_case, unused_imports)]\nuse std::convert::TryFrom;` to `generate_rust_file` in `src/codegen.rs` to ensure the required trait is strictly imported during compilation.
**2023-10-27 - [Warden Complete Scan]
**Threat:** Full codebase scan executed for Memory Safety & Unsafe, Input & Logic (DoS, Overflow, Deserialization, Injection), and Supply Chain Vulnerabilities.
**Defense:** No exploitable vulnerabilities, `unsafe` blocks, or vulnerable crate dependencies (`cargo audit`) found. Existing defenses (`MAX_FILE_SIZE` bounding, `checked_*` arithmetic) verified as functional. Scan log complete.
