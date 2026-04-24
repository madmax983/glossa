## [Splitting The Blob: Conversion]
**Tangle:** `src/semantic/conversion.rs` was a monolithic file (~2800 lines) mixing extraction logic, classification heuristics, and the main statement interpretation loop all in one file, making it very difficult to reason about pattern matching and traversal.
**Blueprint:**
1. Created `src/semantic/conversion/` module.
2. Extracted `extract_*` functions to `src/semantic/conversion/extraction.rs`.
3. Extracted `classify_*` and `try_print_*` functions to `src/semantic/conversion/classification.rs`.
4. Kept the main interpretation facade in `src/semantic/conversion/mod.rs`.
5. Resolved internal visibilities with `pub(crate)` and `pub` according to test requirements.
**Stability:** Significantly improves module cohesion by separating "intent diagnosis" (classification) from "data extraction" (extraction). File sizes are smaller and domains are clearer.

## [Splitting The Blob: Conversion]
**Tangle:** `src/semantic/conversion.rs` was a monolithic file (~2800 lines) mixing extraction logic, classification heuristics, and the main statement interpretation loop all in one file, making it very difficult to reason about pattern matching and traversal.
**Blueprint:**
1. Created `src/semantic/conversion/` module.
2. Extracted `extract_*` functions to `src/semantic/conversion/extraction.rs`.
3. Extracted `classify_*` and `try_print_*` functions to `src/semantic/conversion/classification.rs`.
4. Kept the main interpretation facade in `src/semantic/conversion/mod.rs`.
5. Resolved internal visibilities with `pub(crate)` and `pub` according to test requirements.
**Stability:** Significantly improves module cohesion by separating "intent diagnosis" (classification) from "data extraction" (extraction). File sizes are smaller and domains are clearer.
