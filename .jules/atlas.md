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

## [Encapsulating The Parser]
**Tangle:** The `src/parser` module had the `grammar` module (`grammar.pest` and `grammar.rs`) directly in its root. This allowed other parsing submodules (like `statements`, `expressions`, `declarations`) to be entangled with the low-level parser implementation, breaking strict hierarchical separation and increasing coupling.
**Blueprint:** Created a new `src/parser/core` module. Moved the auto-generated `pest` parser (`GlossaParser`), its `Rule` enum, and the `grammar.pest` definition into `src/parser/core/`. Updated all other submodules to import from `parser::core`, establishing a clear boundary where `core` contains the raw parsing logic and the rest of the `parser` module acts as a high-level AST builder.
**Stability:** Enforced a cleaner hierarchical structure and separation of concerns within the parsing phase.
