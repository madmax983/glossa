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
## [Encapsulating Assembly Inner Workings]
**Tangle:** The `src/semantic/assembly/mod.rs` module was exposed publicly (`pub mod assembly;`) in `src/semantic/mod.rs`, breaking the module boundaries. Moreover, `Assembler` was exported out of `assembly` by `pub use assembly::Assembler`. Then `assembly` module leaked more internal structure.
**Blueprint:**
1. Modified `src/semantic/mod.rs` to restrict the `assembly` module with `pub(crate) mod assembly;`.
2. Changed `use glossa::semantic::assembly::Assembler;` to `use glossa::semantic::Assembler;` inside `src/semantic/assembly/mod.rs` doc tests to adapt to the new visibility restrictions since `Assembler` is exported in `semantic`.
**Stability:** Achieved higher cohesion by keeping the public API surface minimal and ensuring internal structures like `assembly` sub-components don't leak out of their intended domains.
