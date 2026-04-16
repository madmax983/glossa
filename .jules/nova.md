# Nova's Journal - The Idea Graveyard

## 🌟 The Oracle (ὁ Μάντις)
**Concept:** A CLI tool (`glossa explain`) that visualizes the semantic assembly process, showing how Greek morphology (case endings) maps to grammatical slots (Subject, Object, Verb).
**Fate:** Merged
**Lesson:** Visualizing the compiler's internal state is powerful for users learning the language. Also, I discovered that the compiler prioritizes Participle analysis for words ending in `-ον` if they aren't in the lexicon, which creates ambiguity for Accusative Nouns.

## 🌟 The Bard (ὁ Ῥαψῳδός)
**Concept:** A semantic syntax highlighter (`glossa highlight`) that colors code based on morphological analysis (Subject=Blue, Object=Red, etc.) instead of regexes.
**Fate:** Merged
**Lesson:** Visualizing the compiler's understanding reveals subtle analysis behaviors (like "λόγον" being potentially analyzed as an Adjective). It proves the morphological engine is robust enough for reverse-mapping.

## 🌟 The Mentor (ὁ Μέντωρ)
**Concept:** An interactive tutorial mode (`glossa mentor`) that guides users through learning the language via Socratic challenges. It verifies not just syntax but semantic correctness (e.g., "Create a variable named 'x'").
**Fate:** Merged
**Lesson:** By connecting the `Repl` loop with the `Analyzer`'s internal state, we can create a powerful educational tool that provides real-time feedback. This transforms the compiler from a tool into a teacher.

## 🌟 The Alchemist (ὁ Χημικός)
**Concept:** A Python transpiler (`glossa alchemist`) that converts analyzed Glossa programs directly to Python source code, providing a second export target beyond Rust.
**Fate:** Proposed
**Lesson:** Python's dynamic typing and simplicity make it an easy compilation target for Glossa's structural abstractions. Implementing it proved that the semantic assembler phase is decoupled perfectly from the Rust codegen phase.

## 🌟 The Labyrinth (ὁ Λαβύρινθος)
**Concept:** A CLI tool (`glossa labyrinth`) that visualizes the control flow graph of a Glossa program as a Mermaid.js diagram. This expands the "Architectural Transparency" feature set by tracing logic branching instead of just structural relations.
**Fate:** Merged
**Lesson:** Iterating over the complex nested variants in `AnalyzedStatement` proves the semantic AST is stable enough for deep structural introspection. Representing implicit logic branches explicitly via node/edge graph generators reinforces the language's determinism.
## The Papyrus (ὁ Πάπυρος)
**Concept:** A SQL Schema generator (`glossa papyrus`) that transpiles Glossa structs (`εἶδος`) directly to `CREATE TABLE` SQL statements.
**Fate:** Proposed
**Lesson:** Treating Glossa as a Data Definition Language bridges ancient syntax with modern relational databases.
## The Auditor (ὁ Λογιστής)
**Concept:** A basic static analysis tool / linter (`glossa audit`) that traverses the semantic AST (`AnalyzedProgram`) to find code smells, such as unused variables and unnecessary mutable declarations.
**Fate:** Merged
**Lesson:** Iterating over the complex nested variants in `AnalyzedStatement` and `AnalyzedExpr` provides a strong foundation for building static analysis tools without modifying core logic.
## 🌟 The Simulator (ὁ Προσομοιωτής)
**Concept:** A CLI tool (`glossa simulate`) that uses the internal tree-walk interpreter to evaluate Glossa AST nodes directly without full codegen to Rust. It provides a "Debug Mode" to test code correctness dynamically.
**Fate:** Merged
**Lesson:** Providing a raw runtime visualization connects the user more intimately with the code's evaluation semantics, reinforcing language design without the overhead of the Rust toolchain.
