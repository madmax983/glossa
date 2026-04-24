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

## 🌟 The Catalog (ὁ Κατάλογος)
**Concept:** A CLI tool (`glossa catalog`) that lists all built-in verbs, operators, particles, and keywords available in the Glossa language. Grouped by their internal morphology (PartOfSpeech) and nicely formatted in terminal tables using `comfy-table`.
**Fate:** Merged
**Lesson:** Adding an iterator to the static lexicon unlocks great introspection capabilities for tooling, allowing users to browse the exact translation matrix that the semantic engine uses under the hood.

## The Scholar (ὁ Σχολαστικός)
**Concept:** A Markdown documentation generator (`glossa scholar`) that uses the compiler's semantic phase to extract and format types, traits, and functions.
**Fate:** Merged
**Lesson:** Using the `AnalyzedProgram`'s scope directly allows us to generate accurate API documentation effortlessly, proving the power of a centralized semantic model.

## 🌟 The Emissary (ὁ Ἀπεσταλμένος)
**Concept:** A JSON Schema generator (`glossa emissary`) that transpiles Glossa struct definitions (`εἶδος`) directly into JSON Schema representations.
**Fate:** Merged
**Lesson:** Providing a tool that creates JSON Schemas alongside Papyrus's SQL creation bridges ancient Glossa syntax into modern API development ecosystems, proving the framework can decouple easily from strictly code generation into widespread data specification formats.
