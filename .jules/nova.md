# Nova's Journal - The Idea Graveyard

## 🌟 The Oracle (ὁ Μάντις)
**Concept:** A CLI tool (`glossa explain`) that visualizes the semantic assembly process, showing how Greek morphology (case endings) maps to grammatical slots (Subject, Object, Verb).
**Fate:** Merged
**Lesson:** Visualizing the compiler's internal state is powerful for users learning the language. Also, I discovered that the compiler prioritizes Participle analysis for words ending in `-ον` if they aren't in the lexicon, which creates ambiguity for Accusative Nouns.

## 🌟 The Bard (ὁ Ῥαψῳδός)
**Concept:** A semantic syntax highlighter (`glossa highlight`) that colors code based on morphological analysis (Subject=Blue, Object=Red, etc.) instead of regexes.
**Fate:** Merged
**Lesson:** Visualizing the compiler's understanding reveals subtle analysis behaviors (like "λόγον" being potentially analyzed as an Adjective). It proves the morphological engine is robust enough for reverse-mapping.

## 🌟 The Muse (ὁ Μοῦσα)
**Concept:** A generative tool (`glossa muse`) that provides themed code snippets ("Inspirations") to help users learn the language and its mythology.
**Fate:** Merged
**Lesson:** Adding a "Creative Mode" to the CLI makes the learning curve less steep and reinforces the "Code as Myth" philosophy. Also, verified that `syntax` for functions is still evolving (no explicit `fn` keyword in grammar, handled semantically via `ὁρίζειν`).
