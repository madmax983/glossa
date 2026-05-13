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

## [Splitting The Blob: Lexicon and Conversion]
**Tangle:** `src/morphology/lexicon.rs` and `src/semantic/conversion.rs` had grown into large 'Blob' files (>2000 lines), mixing massive static datasets (`LEXICON`), extensive logic, and huge inline test suites.
**Blueprint:**
1. Created `src/morphology/lexicon/` and `src/semantic/conversion/` module directories.
2. Extracted the massive `LEXICON` static mapping into `src/morphology/lexicon/data.rs`.
3. Moved large inline tests (`mod tests`) into dedicated `src/morphology/lexicon/tests.rs` and `src/semantic/conversion/tests.rs` files, exposing them via `#[cfg(test)] mod tests;`.
**Stability:** Significantly improved readability and navigation by flattening out test suites and static maps, adhering to the "file count low, module flat" architecture.
