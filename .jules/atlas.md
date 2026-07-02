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

## [Encapsulating Tools with a Facade]
**Tangle:** The `src/tools/` directory exposed its internal modules (`runner`, `tester`, `interpreter`, etc.) as `pub mod`. This breaks encapsulation and allows external code (or the `src/main.rs` binary) to depend tightly on the internal hierarchy of the tools ecosystem.
**Blueprint:**
1. Modified `src/tools/mod.rs` to mark all submodules (e.g., `cli`, `runner`, `repl`) as `pub(crate) mod`.
2. Created a public Facade in `src/tools/mod.rs` using `pub use` to selectively re-export only the exact structs and functions the `glossa` binary target needs.
3. Updated `src/main.rs` and all integration tests to import directly from the Facade (`glossa::tools::*`) rather than the inner modules.
**Stability:** Enforces high cohesion and strictly defines the boundary between the compiler library (`src/lib.rs`) and the binary (`src/main.rs`), eliminating the "Leaky Abstraction".
