1. **Create an ADR for the Facade Pattern in Tools Module.**
   - Since Atlas refactored `src/tools/mod.rs` to enforce the Facade pattern across almost all tool submodules (`cli`, `dictionary`, `tester`, `runner`, `repl`, `narrator`, `alchemist`, `auditor`, etc.) by changing their visibility to `pub(crate) mod`, and ADR 016 only specifically talks about `report` and `ui`, I will create a new ADR (e.g. `docs/adr/020-enforce-facade-pattern-in-tools.md`) to explicitly record this architectural decision.
   - This ADR will follow the standard Nygard template (Title, Status, Context, Decision, Consequences) to document *why* this change was made (to prevent leaking implementation details and create a cleaner, flattened public API).

2. **Update the Architecture Diagrams (Mermaid).**
   - Review `docs/architecture.md`.
   - The current Container diagram (C4 Level 2) correctly shows all these tools nested within the "Developer Experience (Nova)" boundary. The arrows point correctly.
   - The encapsulation is mostly a module-level boundary concern. If there's any specific relation to remove, I will check. But given `src/tools/mod.rs` acts as a facade, the C4 Component diagram of the Tools could be updated if it existed. The existing diagram is just a C4 Container list of tools inside a boundary. The current boundary nicely represents the facade.
   - Therefore, the visual update is likely minimal, but I should ensure the descriptions in `docs/architecture.md` are aligned with the new structure if anything is glaringly missing. Wait, the system doesn't require me to change `docs/architecture.md` *unless* the structure actually changed in a way that affects the C4 model. However, since the internal tools are now technically behind a Facade (the `tools` module itself), I could consider adding a C4 Component diagram for the `Tools` module to show the Facade, or I'll just state that the C4 Container diagram already handles the grouping nicely.
   - To be safe, I'll update the Container Diagram slightly to explicitly mention the facade in the boundary description or similar. Actually, the C4 Container Diagram already lists `tools` as a `Container_Boundary`. I will add a note about the Facade in the description.

3. **Complete pre-commit steps.**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

4. **Submit the pull request.**
   - I will use the Codex persona format to submit the changes.
   - Title: `📜 Codex: ADR 020 & Architecture Diagram Update`
   - Description with `🧠 **Decision:**`, `🗺️ **Visuals:**`, and `🔗 **Link:**`.
