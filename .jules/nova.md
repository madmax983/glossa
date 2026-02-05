# Nova's Journal - The Idea Graveyard

## 🌟 The Oracle (ὁ Μάντις)
**Concept:** A CLI tool (`glossa explain`) that visualizes the semantic assembly process, showing how Greek morphology (case endings) maps to grammatical slots (Subject, Object, Verb).
**Fate:** Merged
**Lesson:** Visualizing the compiler's internal state is powerful for users learning the language. Also, I discovered that the compiler prioritizes Participle analysis for words ending in `-ον` if they aren't in the lexicon, which creates ambiguity for Accusative Nouns.

## 🌟 The Rhapsode (ὁ Ῥαψῳδός)
**Concept:** An HTML export tool (`glossa export`) that generates an interactive "tapestry" of the code. Users can hover over words to see their grammatical role and morphological analysis, rendered with beautiful typography and CSS syntax highlighting.
**Fate:** Merged
**Lesson:** Visualizing semantic roles required bridging the gap between the AST (structure) and the Assembler (semantic meaning), which lose word order information. By creating a "pool" of assembled constituents and consuming them as we traverse the AST, we can reconstruct the semantic coloring for the original source code.
