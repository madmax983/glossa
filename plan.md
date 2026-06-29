1. **Create ADRs for the missing tools:**
   - I will use the `run_in_bash_session` tool to execute the following command:
     ```bash
     cat << 'INNER_EOF' > docs/adr/022-gnomon-complexity-estimator.md
     # 022. Gnomon Complexity Estimator

     Date: 2026-05-10

     ## Status

     Proposed

     ## Context

     Developers needed a way to statically estimate the Big-O time complexity of ΓΛΩΣΣΑ programs before running them. The existing tools did not provide loop depth analysis or complexity estimation.

     ## Decision

     We added the "Gnomon" tool (`src/tools/gnomon.rs`) to the Developer Experience (Nova) suite. It traverses the Abstract Syntax Tree to calculate loop depth and estimates execution time complexity.

     ## Consequences

     - Provides static complexity analysis for developers.
     - Increases the utility of the Nova tool suite.
     - Adds complexity to the compiler's toolchain that must be maintained.
     INNER_EOF

     cat << 'INNER_EOF' > docs/adr/023-haruspex-ast-visualizer.md
     # 023. Haruspex AST Visualizer

     Date: 2026-05-10

     ## Status

     Proposed

     ## Context

     While the Cartographer maps architecture and the Labyrinth traces control flow, compiler developers lacked a tool to inspect the raw semantic tree structure, seeing exactly how expressions are nested and typed.

     ## Decision

     We added the "Haruspex" tool (`src/tools/haruspex.rs`) to inspect the semantic AST (`AnalyzedProgram`) of a ΓΛΩΣΣΑ program and translate it into a DOT graph for visualization with Graphviz.

     ## Consequences

     - Developers can easily visualize and debug the semantic AST.
     - Requires Graphviz as an external dependency for rendering DOT graphs.
     INNER_EOF

     cat << 'INNER_EOF' > docs/adr/024-scholar-api-doc-generator.md
     # 024. Scholar API Doc Generator

     Date: 2026-05-10

     ## Status

     Proposed

     ## Context

     There was a gap between raw semantic analysis and human-readable references. Developers had to read Ancient Greek source files to understand an API's shape, which hindered usability.

     ## Decision

     We implemented the "Scholar" tool (`src/tools/scholar.rs`) to bridge this gap. It extracts type definitions, traits, and functions from the parsed ΓΛΩΣΣΑ program and formats them into standardized GitHub-flavored Markdown.

     ## Consequences

     - Automatically generated API documentation improves developer experience.
     - Adds a new parsing layer to extract doc comments and structural definitions.
     INNER_EOF
     ```

2. **Verify ADR creation:**
   - I will use the `run_in_bash_session` tool to run `ls -la docs/adr/` and `cat docs/adr/022-gnomon-complexity-estimator.md` to verify the creation and contents of the new ADR files.

3. **Update `docs/architecture.md`:**
   - I will use the `run_in_bash_session` tool to execute a Python patch script. It will run the following commands:
     ```bash
     cat << 'INNER_EOF' > patch.py
     with open('docs/architecture.md', 'r') as f:
         content = f.read()

     target1 = '        Container(dictionary, "The Lexicon", "src/tools/dictionary.rs", "The Source of Truth for Words (Dictionary)")'
     replacement1 = '        Container(dictionary, "The Lexicon", "src/tools/dictionary.rs", "The Source of Truth for Words (Dictionary)")\n        Container(gnomon, "Gnomon", "src/tools/gnomon.rs", "Big-O Complexity Estimator")\n        Container(haruspex, "Haruspex", "src/tools/haruspex.rs", "Graphviz AST Visualizer")\n        Container(scholar, "Scholar", "src/tools/scholar.rs", "Markdown API Documentation Generator")'

     target3 = '    Rel(semantic, papyrus, "Analyzed Program")'
     replacement3 = '    Rel(semantic, papyrus, "Analyzed Program")\n    Rel(semantic, gnomon, "Analyzed Program")\n    Rel(semantic, haruspex, "Analyzed Program")\n    Rel(semantic, scholar, "Analyzed Program")'

     content = content.replace(target1, replacement1)
     content = content.replace(target3, replacement3)

     with open('docs/architecture.md', 'w') as f:
         f.write(content)
     INNER_EOF
     python3 patch.py
     rm patch.py
     ```

4. **Verify `docs/architecture.md` update:**
   - I will use the `run_in_bash_session` tool to run `cat docs/architecture.md | grep -A 30 "Developer Experience"` and `cat docs/architecture.md | grep -A 30 "Rel(semantic, alchemist"` to verify that the Mermaid diagram was correctly updated.

5. **Test the changes:**
   - I will use the `run_in_bash_session` tool to run `cargo test --all-features` to ensure no regressions were introduced.

6. **Complete Pre-Commit Steps:**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Submit PR:**
   - I will submit the PR using the `submit` tool with Title: `📜 Codex: ADR 022-024 & Architecture Diagram Update`, passing the description directly to the tool's parameter:
   ```markdown
   🧠 **Decision:** Drafted ADRs to record the addition of the Gnomon, Haruspex, and Scholar tools to the Developer Experience (Nova) tool suite.
   🗺️ **Visuals:** Updated the Container Diagram in `docs/architecture.md` to map the new components (`gnomon`, `haruspex`, `scholar`) and their relationships with the Semantic Analyzer.
   🔗 **Link:** See `docs/adr/022-gnomon-complexity-estimator.md`, `docs/adr/023-haruspex-ast-visualizer.md`, and `docs/adr/024-scholar-api-doc-generator.md`.
   ```
