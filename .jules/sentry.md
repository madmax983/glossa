## 2024-05-22 - [Morphological Ambiguity]
**Learning:** `analyze_noun` relies on a fixed check order (Third -> Second -> First), which causes `First Declension Alpha` nouns (ending in -α) to be misidentified as `Second Declension Neuter` plurals (also ending in -α) if they don't have a lexicon entry. `analyze_noun_all` correctly identifies both but requires the caller to disambiguate.
**Action:** Use `analyze_noun_all` when dealing with ambiguous endings or ensure the lexicon is populated. Tests for ambiguous words must verify the *presence* of the correct analysis, not just the first one.
