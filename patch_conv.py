import sys
with open(".jules/razor.md", "a") as f:
    f.write("\n## [Reduction]\n**Bloat:** Complex `DoubleSubject` and Missing Verb state checks spread out in `Assembler::finalize()` which were bypassed by certain verbs and nested phrases.\n**Cut:** Removed duplicate and misfiring checks in `finalize()`. Placed a single unified `DoubleSubject` check at the beginning of `classify_expression` in `src/semantic/conversion.rs`.\n**Saved:** Avoided messy verb classification bypassing and consolidated grammatical validation to where semantic structure is actually clear.\n")
