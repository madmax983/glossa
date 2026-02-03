## [Sentence Diagrammer]
**Concept:** Added `glossa diagram` command that outputs Mermaid flowcharts visualizing the semantic structure (Subject, Verb, Object) of Greek sentences.
**Fate:** Merged
**Lesson:** Visualizing the internal state of the "Assembler" (the slot-based parser) is a great way to debug and explain the "Greek-coding" concept. However, I discovered that the morphology engine sometimes aggressively classifies unknown words as Participles if they have common endings, which made testing tricky.
