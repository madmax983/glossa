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
**Tangle:** `src/semantic/conversion.rs` was a monolithic file (~2880 lines) containing logic for extracting values, processing variable bindings, handling functions, assertions, prints, and collections.
**Blueprint:**
1. Created `src/semantic/conversion/` module.
2. Extracted value logic to `src/semantic/conversion/values.rs`.
3. Extracted binding logic to `src/semantic/conversion/bindings.rs`.
4. Extracted assertions to `src/semantic/conversion/assertions.rs`.
5. Extracted function calls to `src/semantic/conversion/functions.rs`.
6. Extracted print logic to `src/semantic/conversion/prints.rs`.
7. Extracted collections logic to `src/semantic/conversion/collections.rs`.
8. Retained core routing logic in `src/semantic/conversion/mod.rs`.
**Stability:** Significantly improved structural organization by decoupling distinct semantic processing rules, adhering strictly to the Single Responsibility Principle, and eliminating the file bloat.
