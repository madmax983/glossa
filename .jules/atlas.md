## [Splitting The Blob: Assembler]
**Tangle:** `src/semantic/assembly.rs` was a monolithic file (~2000 lines) mixing DTOs (`AssembledStatement`, `Constituent`) with complex assembly logic (`Assembler`) and tests.
**Blueprint:**
1. Created `src/semantic/assembly/` module.
2. Extracted DTOs to `src/semantic/assembly/model.rs`.
3. Moved logic to `src/semantic/assembly/mod.rs`.
4. Updated dependent modules to import from `crate::semantic::assembly`.
**Stability:** Improves separation of concerns (Data vs Logic) and reduces file size.

## [Splitting The Blob: Assembly]
**Tangle:** `src/semantic/assembly.rs` was a monolithic file (~2000 lines) mixing DTOs (`AssembledStatement`, `Constituent`) with complex assembly logic (`Assembler`) and tests.
**Blueprint:**
1. Created `src/semantic/assembly/` module.
2. Extracted DTOs to `src/semantic/assembly/model.rs`.
3. Moved logic to `src/semantic/assembly/mod.rs`.
4. Updated dependent modules to import from `crate::semantic::assembly`.
**Stability:** Improves separation of concerns (Data vs Logic) and reduces file size.

## [Encapsulating Internal Modules]
**Tangle:** Several modules under `src/tools/` (specifically `cache`, `report`, and `ui`) and `src/semantic/assembly/` (`model`) were exposed as `pub mod`, breaking encapsulation by exposing internal implementation details to the public API.
**Blueprint:** Modified `src/tools/mod.rs` and `src/semantic/assembly/mod.rs` to restrict these modules with `pub(crate) mod`.
**Stability:** Achieved higher cohesion by keeping the public API surface minimal and ensuring internal structures don't leak out of their intended domains.

## [Splitting The Blob: Conversion]
**Tangle:** `src/semantic/conversion.rs` was a monolithic file (~3000 lines) mixing extraction logic, classification logic, orchestration, and extensive tests, creating a "Blob" anti-pattern that was difficult to navigate and maintain.
**Blueprint:**
1. Converted `src/semantic/conversion.rs` into a module directory `src/semantic/conversion/`.
2. Extracted extraction logic into `src/semantic/conversion/extract.rs`.
3. Extracted classification logic into `src/semantic/conversion/classify.rs`.
4. Extracted tests into `src/semantic/conversion/tests.rs`.
5. Exposed necessary helper functions via `pub(crate)` to maintain visibility boundaries while allowing tests to compile.
**Stability:** Significantly reduced file sizes, improved separation of concerns, and maintained strict `pub(crate)` encapsulation.
