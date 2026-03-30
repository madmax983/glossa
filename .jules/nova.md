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

## 🌟 The Arena (ἡ Ἀρένα)
**Concept:** A Terminal User Interface (TUI) tool (`glossa arena`) built with `ratatui` that allows users to interactively view the syntax-highlighted source code, the generated Rust code, and the semantic "Mosaic" side-by-side.
**Fate:** Merged
**Lesson:** Adding a TUI makes visualizing the compiler's intermediate representations much more immersive. `ratatui` integrates nicely but requires managing terminal state properly (raw mode, alternate screens) to avoid messing up the user's terminal.
