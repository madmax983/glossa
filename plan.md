1. **Draft ADR `docs/adr/021-enforce-facade-pattern.md`**
   - Title: Enforcing the Facade Pattern in src/lib.rs
   - Status: Accepted
   - Context: The `glossa` crate previously exposed all its internal modules (`ast`, `codegen`, `limits`, `morphology`, `parser`, `semantic`, `text`) as fully public (`pub mod`), leaking implementation details and creating a sprawling API.
   - Decision: Refactored `src/lib.rs` to change these modules to `pub(crate) mod` and added explicit `pub use` statements for the true public API (`ast::Program`, `codegen::generate_rust`, `parser::parse`, `semantic::{AnalyzedProgram, analyze_program}`).
   - Consequences: This creates a clean "Facade" that hides messy internal sub-modules while exposing only what the user needs.
2. **Update `docs/architecture.md`**
   - Add a new section `## Public API (Facade Pattern)` with a Mermaid Class Diagram illustrating the Facade pattern. The diagram should show the external consumer accessing only the Facade (`glossa::lib`), which then delegates to the internal, hidden submodules (`parser`, `semantic`, `codegen`, `ast`).
   - Add `Catalog` to `docs/architecture.md` if it isn't there already (wait, it's already in the C4 container diagram as "Container(catalog, ...)").
3. **Run Pre-Commit Checks**
   - Call `pre_commit_instructions` tool to run verification.
4. **Submit**
   - Title: "📜 Codex: Enforce Facade Pattern ADR & Architecture Diagram Update"
   - Description formatted as required by Codex.
