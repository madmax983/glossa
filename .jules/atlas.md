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

## [Encapsulating Assembly Module]
**Tangle:** `src/semantic/assembly/` was exposed as a public module (`pub mod assembly`) when its internal struct models are meant to be an implementation detail hidden behind the facade at `src/semantic/mod.rs`.
**Blueprint:** Modified `src/semantic/mod.rs` to restrict `assembly` to `pub(crate) mod assembly`, routing all test and internal imports through the facade export `glossa::semantic::*`.
**Stability:** Achieves stricter boundary enforcement and hides the inner `assembly` directory layout from public consumers.
