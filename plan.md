1. **Detect Unrecorded Tool**:
   - We observed that `src/tools/gnomon.rs` (The Gnomon - Big-O Complexity Estimator) is present in the codebase.
   - However, `docs/architecture.md` and `docs/adr/` lack any record of this tool, which violates the Codex principles of Architectural Transparency.
2. **Draft the ADR**:
   - Create `docs/adr/022-gnomon-complexity-estimator.md`.
   - Title: "022. Add Gnomon to Developer Experience Tools"
   - Status: Proposed
   - Date: 2026-08-12 (Current Date)
   - Context: Explain the addition of `src/tools/gnomon.rs` (Big-O Complexity Estimator).
   - Decision: Formally recognize Gnomon within the Developer Experience (Nova) toolset.
   - Consequences: Adds a static analysis tool for complexity; requires maintaining the AST traversal.
3. **Update Architecture Map**:
   - Update `docs/architecture.md` C4 Container diagram to include Gnomon.
     - `Container(gnomon, "Gnomon", "src/tools/gnomon.rs", "Estimates Big-O complexity from loop depth")`
   - Add a relationship from Semantic to Gnomon.
     - `Rel(semantic, gnomon, "Analyzed Program")`
4. **Pre-commit Instructions**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. **Submit the Change**:
   - PR Title: `📜 Codex: ADR 022 & Architecture Diagram Update for Gnomon`
   - Description format requested by the Codex persona:
     * 🧠 **Decision:** Recorded the addition of `gnomon` tool to Developer Experience toolset.
     * 🗺️ **Visuals:** Updated C4 Container diagram to show Gnomon.
     * 🔗 **Link:** See `docs/adr/022-gnomon-complexity-estimator.md`.
