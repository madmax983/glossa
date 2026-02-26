# Echo's DX Audit Report 🗣️

**Date:** 2024-05-23
**Auditor:** Echo (The Voice of the User)

## Summary
The "Quick Start" experience is largely successful. The provided examples in `README.md` and `examples/quickstart.γλ` are functional and consistent. However, several documentation gaps and minor inconsistencies were found.

## Findings

### ✅ Successes
*   **Runnable Examples:** `examples/quickstart.γλ` compiles and runs correctly.
*   **Valid Documentation:** The code snippets in `README.md`, when combined, form a valid and executable program.
*   **Helpful Tools:** `bard` (The Narrator) provides excellent, readable explanations of the code logic. `mosaic` offers a useful structural breakdown.

### ⚠️ Issues & Friction Points

1.  **Missing CLI Documentation:**
    *   The module-level documentation in `src/tools/cli.rs` lists `run`, `build`, `check`, `highlight`, `lookup`, `bard`, and `repl`.
    *   **It misses:** `test`, `map`, and `mosaic` (though `mosaic` and `map` are feature-gated, they should be documented).

2.  **Tool Limitations:**
    *   **Map Tool:** `glossa map examples/quickstart.γλ` produces an empty Mermaid diagram, despite the file containing a `struct` definition. This appears to be a bug or severe limitation.
    *   **Bard Transliteration:** The `bard` tool strips accents from Greek identifiers in its output (e.g., `ὄνομα` becomes `ονομα`). This may be confusing for users expecting exact matches.

3.  **File Extension Inconsistency:**
    *   The project uses both `.gl` (e.g., `my_tests.gl`) and `.γλ` (e.g., `quickstart.γλ`).
    *   The README refers to `my_tests.γλ`, but the file is actually `my_tests.gl`.

4.  **Implicit Loop Variable:**
    *   The loop syntax `διὰ collection, variable action` implicitly declares `variable`. The README example uses a variable declared *outside* the loop in a previous step (`ξ`), which might mislead users into thinking they must declare it first.

## Recommendations
1.  **Update CLI Docs:** specificallly `src/tools/cli.rs` to include all available commands.
2.  **Fix Map Tool:** Investigate why `map` returns empty diagrams for simple structs.
3.  **Standardize Extensions:** Pick one extension (preferably `.γλ` for the brand, or `.gl` for ease of typing) and stick to it in docs and examples.
4.  **Clarify Loop Syntax:** Update docs to explicitly state that the loop variable is a new binding for the loop scope.
