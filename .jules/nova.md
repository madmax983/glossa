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

## 🌟 The Simulator (ὁ Ὑποκριτής)
**Concept:** A CLI tool (`glossa simulate`) that allows executing ΓΛΩΣΣΑ programs directly via the internal tree-walk interpreter without compiling to Rust or invoking `rustc`.
**Fate:** Merged
**Lesson:** Interpreting the semantic AST directly provides immediate execution feedback and skips `rustc` compilation overhead, improving the prototyping loop dramatically. It proves the value of the `Interpreter` module beyond just internal testing.
