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
