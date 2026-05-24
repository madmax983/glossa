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
**[Atlas Module Structure Fix]
**Tangle:** Several experimental and internal utility tools in `src/tools/` (e.g., `alchemist`, `auditor`, `papyrus`, `weave`, etc.) are defined with `pub mod` in `src/tools/mod.rs`, but many internal helper functions inside them do not need to be part of the public API surface of the `glossa` crate. Additionally, some modules like `tester` and `ui` should likely be restricted. However, restricting them breaks existing integration tests in the `tests/` folder since Rust requires `pub` for external tests to access them.
**Blueprint:** Keep `pub mod` for `tools` items since they are consumed by the `tests/` crate and the `glossa` binary (which acts as an external crate to the library). The current structure with `pub mod` for tools in `src/tools/mod.rs` and `pub mod` for semantic submodules is necessary due to the testing and binary structure of the repository. No structural changes are needed at this time.
