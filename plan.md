1.  **Draft ADR for Gnomon:**
    - Use `run_in_bash_session` to run `cat << 'EOF' > docs/adr/022-gnomon-complexity-estimator.md` followed by the markdown content:
      ```markdown
      # 022. Add Gnomon (ὁ Γνώμων) Big-O Complexity Estimator

      Date: 2026-08-19
      Status: Proposed

      ## Context
      As programs written in ΓΛΩΣΣΑ become more complex, developers need tools to understand their performance characteristics statically. The "Gnomon" tool (`src/tools/gnomon.rs`) was introduced to estimate the Big-O time complexity by statically analyzing loop depth (e.g., `while` and `for` loops) in the semantic AST. However, this addition to the "Developer Experience (Nova)" toolset is currently unrecorded in our architecture decision records, violating the principle of Architectural Transparency.

      ## Decision
      We formally recognize the **Gnomon** tool as a component within the "Developer Experience (Nova)" container in `src/tools/`.
      Gnomon is defined as the Big-O complexity estimator that takes an `AnalyzedProgram` from the Semantic Analyzer, traverses its abstract syntax tree to calculate loop depth, and reports an estimated time complexity (e.g., O(1), O(N), O(N^2)).

      ## Consequences
      - **Positive:** Improved architectural clarity. Developers have a static analysis tool for estimating program complexity without running code.
      - **Negative:** Increased maintenance surface. Changes to control flow structures in the semantic AST will require updates to the Gnomon visitor logic.
      ```
    - Use `run_in_bash_session` with `cat docs/adr/022-gnomon-complexity-estimator.md` to verify the file creation.

2.  **Draft ADR for Haruspex:**
    - Use `run_in_bash_session` to run `cat << 'EOF' > docs/adr/023-haruspex-ast-visualizer.md` followed by the markdown content:
      ```markdown
      # 023. Add Haruspex (ὁ Ἱεροσκόπος) Graphviz AST Visualizer

      Date: 2026-08-19
      Status: Proposed

      ## Context
      Compiler development requires deep inspection of intermediate representations. While existing tools map high-level architecture (Cartographer) or control flow (Labyrinth), there was a missing capability to visualize the raw, nested structure and types of the semantic Abstract Syntax Tree (`AnalyzedProgram`). The "Haruspex" tool (`src/tools/haruspex.rs`) was built to translate the AST into a DOT graph for Graphviz, but this structural decision remains undocumented.

      ## Decision
      We formally add the **Haruspex** tool to the architectural records as a component of the "Developer Experience (Nova)" container (`src/tools/`).
      Haruspex is defined as the raw semantic AST visualization tool, converting `AnalyzedProgram` structures into Graphviz DOT format to expose precise nesting and type information.

      ## Consequences
      - **Positive:** Enhanced debuggability for compiler developers who can now visualize the exact shape of the semantic tree.
      - **Negative:** Haruspex is tightly coupled to the structure of `AnalyzedExpr` and `AnalyzedStatement`. Any changes to the semantic model require immediate updates to the DOT generation logic.
      ```
    - Use `run_in_bash_session` with `cat docs/adr/023-haruspex-ast-visualizer.md` to verify the file creation.

3.  **Draft ADR for Scholar:**
    - Use `run_in_bash_session` to run `cat << 'EOF' > docs/adr/024-scholar-api-generator.md` followed by the markdown content:
      ```markdown
      # 024. Add Scholar (ὁ Σχολαστικός) API Doc Generator

      Date: 2026-08-19
      Status: Proposed

      ## Context
      To ensure ΓΛΩΣΣΑ code can be documented effectively without forcing users to read raw source files, an automated documentation tool is needed. The "Scholar" tool (`src/tools/scholar.rs`) was introduced to bridge the gap between the semantic AST and human-readable references by extracting structures (`εἴδη`), traits (`χαρακτῆρες`), and functions (`ἔργα`) into a GitHub-flavored Markdown file (`doc.md`). This new component must be documented to maintain architectural transparency.

      ## Decision
      We formally acknowledge the **Scholar** tool as part of the "Developer Experience (Nova)" container (`src/tools/`).
      Scholar is responsible for generating Markdown API documentation by consuming the `AnalyzedProgram` from the Semantic Analyzer.

      ## Consequences
      - **Positive:** Aligns with documentation-as-code principles. Provides a standardized way to generate human-readable API references.
      - **Negative:** The tool must continuously adapt to new language features (like new types of declarations) to ensure the generated documentation remains complete.
      ```
    - Use `run_in_bash_session` with `cat docs/adr/024-scholar-api-generator.md` to verify the file creation.

4.  **Accept Proposed ADRs older than 7 days:**
    - Use `run_in_bash_session` with `sed -i 's/Status: Proposed/Status: Accepted/' docs/adr/016-alchemist-and-weave.md`
    - Use `run_in_bash_session` with `sed -i 's/Status: Proposed/Status: Accepted/' docs/adr/017-labyrinth-control-flow-graph.md`
    - Check the changed files using `run_in_bash_session` with `grep -H "Status:" docs/adr/016-alchemist-and-weave.md docs/adr/017-labyrinth-control-flow-graph.md`

5.  **Update Architecture Diagram:**
    - Use `replace_with_git_merge_diff` to edit `docs/architecture.md`. Add the missing tools to the "Developer Experience (Nova)" container block:
      ```
      <<<<<<< SEARCH
              Container(dictionary, "The Lexicon", "src/tools/dictionary.rs", "The Source of Truth for Words (Dictionary)")
              Container(highlight, "Highlighter", "src/tools/highlight.rs", "Semantic syntax highlighting")
              Container(interpreter, "Interpreter", "src/tools/interpreter.rs", "In-memory tree-walk simulator")
              Container(labyrinth, "Labyrinth", "src/tools/labyrinth.rs", "Visualizes the control flow graph as a Mermaid flowchart")
              Container(mentor, "Mentor", "src/tools/mentor.rs", "Interactive Tutorial Mode")
              Container(mosaic, "Mosaic", "src/tools/mosaic.rs", "Visualizes Semantic Assembly")
              Container(narrator, "The Bard", "src/tools/narrator.rs", "Generates English narrative ('Scroll of Logic') from AST")
      =======
              Container(dictionary, "The Lexicon", "src/tools/dictionary.rs", "The Source of Truth for Words (Dictionary)")
              Container(gnomon, "The Gnomon", "src/tools/gnomon.rs", "Estimates Big-O complexity from loop depth")
              Container(haruspex, "The Haruspex", "src/tools/haruspex.rs", "Generates Graphviz DOT representation of AST")
              Container(highlight, "Highlighter", "src/tools/highlight.rs", "Semantic syntax highlighting")
              Container(interpreter, "Interpreter", "src/tools/interpreter.rs", "In-memory tree-walk simulator")
              Container(labyrinth, "Labyrinth", "src/tools/labyrinth.rs", "Visualizes the control flow graph as a Mermaid flowchart")
              Container(mentor, "Mentor", "src/tools/mentor.rs", "Interactive Tutorial Mode")
              Container(mosaic, "Mosaic", "src/tools/mosaic.rs", "Visualizes Semantic Assembly")
              Container(narrator, "The Bard", "src/tools/narrator.rs", "Generates English narrative ('Scroll of Logic') from AST")
      >>>>>>> REPLACE
      <<<<<<< SEARCH
              Container(repl, "REPL", "src/tools/repl.rs", "Interactive Read-Eval-Print Loop")
              Container(report, "Reporter", "src/tools/report.rs", "Generates statistics and structured reports")
              Container(runner, "Runner", "src/tools/runner.rs", "Orchestrates the compilation pipeline")
              Container(scholar, "The Scholar", "src/tools/scholar.rs", "API Doc Generator")
              Container(tester, "The Judge", "src/tools/tester.rs", "Verifies Correctness (Test Runner)")
      =======
              Container(repl, "REPL", "src/tools/repl.rs", "Interactive Read-Eval-Print Loop")
              Container(report, "Reporter", "src/tools/report.rs", "Generates statistics and structured reports")
              Container(runner, "Runner", "src/tools/runner.rs", "Orchestrates the compilation pipeline")
              Container(scholar, "The Scholar", "src/tools/scholar.rs", "API Doc Generator")
              Container(tester, "The Judge", "src/tools/tester.rs", "Verifies Correctness (Test Runner)")
      >>>>>>> REPLACE
      <<<<<<< SEARCH
          Rel(semantic, alchemist, "Analyzed Program")
          Rel(semantic, auditor, "Analyzed Program")
          Rel(semantic, labyrinth, "Analyzed Program")
          Rel(semantic, weave, "Analyzed Program")
          Rel(semantic, papyrus, "Analyzed Program")
          Rel(semantic, codegen, "Analyzed Program")
      =======
          Rel(semantic, alchemist, "Analyzed Program")
          Rel(semantic, auditor, "Analyzed Program")
          Rel(semantic, gnomon, "Analyzed Program")
          Rel(semantic, haruspex, "Analyzed Program")
          Rel(semantic, labyrinth, "Analyzed Program")
          Rel(semantic, scholar, "Analyzed Program")
          Rel(semantic, weave, "Analyzed Program")
          Rel(semantic, papyrus, "Analyzed Program")
          Rel(semantic, codegen, "Analyzed Program")
      >>>>>>> REPLACE
      ```
      (Wait, scholar isn't in the original block so I will append it before `tester`). Let's fix that.
    - Actually, I will use `run_in_bash_session` to do the replacement of `docs/architecture.md` properly.

6.  **Code formatting & Testing:**
    - Use `run_in_bash_session` to run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
    - Although the changes are markdown only, we must verify the project.

7.  **Pre-commit steps:**
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

8.  **Submit PR:**
    - Use `submit` to create a PR with the title `📜 Codex: Documentation for Gnomon, Haruspex, and Scholar` and description:
      ```
      🧠 **Decision:** Recorded the inclusion of Gnomon, Haruspex, and Scholar in the Developer Experience (Nova) toolset. Accepted older proposed ADRs 016 and 017.
      🗺️ **Visuals:** Updated the C4 Container diagram in `docs/architecture.md` to include Gnomon, Haruspex, and Scholar as dependents on the Semantic Analyzer.
      🔗 **Link:** See `docs/adr/022-gnomon-complexity-estimator.md`, `docs/adr/023-haruspex-ast-visualizer.md`, and `docs/adr/024-scholar-api-generator.md`.
      ```
