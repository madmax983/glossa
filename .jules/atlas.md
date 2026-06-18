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
**Tangle:** The internal module `src/semantic/assembly/mod.rs` was exposed to the public API as `pub mod assembly` inside `src/semantic/mod.rs`. This leaked the internal module hierarchy and allowed multiple files to bypass the Facade and use paths like `crate::semantic::assembly::ParticipleConstituent` directly, increasing coupling.
**Blueprint:**
1. Modified `src/semantic/mod.rs` to restrict the module with `pub(crate) mod assembly;`.
2. Updated callers across `src/semantic/` and `src/tools/` to rely solely on the re-exported types from the Facade (`crate::semantic::ParticipleConstituent` etc.), eliminating direct dependence on the internal structure.
**Stability:** Achieved higher encapsulation by ensuring the internal module structure is hidden while providing a clean public interface via the Facade pattern.
