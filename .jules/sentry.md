## [Morphological Ambiguity]
**Learning:** `analyze_noun` relies on a fixed check order (Third -> Second -> First), which causes `First Declension Alpha` nouns (ending in -α) to be misidentified as `Second Declension Neuter` plurals (also ending in -α) if they don't have a lexicon entry. `analyze_noun_all` correctly identifies both but requires the caller to disambiguate.
**Action:** Use `analyze_noun_all` when dealing with ambiguous endings or ensure the lexicon is populated. Tests for ambiguous words must verify the *presence* of the correct analysis, not just the first one.

## [Panic Safety in Morphology Analysis]
**Learning:** `analyze_all` in `src/morphology/mod.rs` was sorting analyses by confidence using `unwrap()` on `partial_cmp`. If `confidence` is `NaN`, this causes a panic.
**Action:** Replaced `unwrap()` with `unwrap_or(std::cmp::Ordering::Equal)` to handle NaN safely. Always verify `partial_cmp` on floats is handled safely.

## [State Modification in Condition Evaluation]
**Learning:** In `src/semantic/assembler.rs`, `check_method_verbs` was popping from `pending_literals` inside a complex `if` condition using `&&`. If subsequent conditions (like `pending_subject` check) failed, the literal was already consumed and lost, causing a logic bug.
**Action:** Always verify all preconditions (using `peek` or `last()`) before performing state-modifying operations (like `pop()`), especially in `if` conditions.

## [Grammar] Person Agreement Check
**Learning:** The assembler was checking Number agreement but missing Person agreement, allowing incorrect sentences like "I (1st) says (3rd)".
**Action:** Added `person` field to `Constituent` and updated `finalize` to check `subj_person != verb_person`. Nouns default to 3rd person.

## [Silent Token Swallowing in Special Checks]
**Learning:** Functions like `check_method_verbs` and `check_special_properties` were returning `true` (indicating "handled") even when they failed to match the full pattern (e.g., missing subject), causing tokens like "split" or "length" to be silently ignored instead of falling back to normal verb/noun processing.
**Action:** Ensure that special handling functions only return `true` when they *successfully* handle the token. If prerequisites aren't met, return `false` to allow fallback to standard processing.

## [Resource Exhaustion (DoS) in Assembler]
**Learning:** The `Assembler` was using unbounded `Vec::push()` for adjectives, literals, and other constituents. This allowed malicious input (e.g., infinite adjectives) to cause OOM panics or Denial of Service.
**Action:** Implemented strict resource limits (`MAX_ADJECTIVES`, etc.) and a new `LimitExceeded` error. Always validate collection sizes before pushing, especially in assemblers/parsers exposed to user input.

## [Silent Token Swallowing in Assembler Fallback]
**Learning:** In `src/semantic/assembler.rs`, the fallback logic for unknown case tokens was silently swallowing tokens if the object slot was already full. It would attempt to set `state.object`, find it occupied, and then do nothing (returning `Ok(())`), leading to data loss.
**Action:** Changed fallback logic to return `Err(AssemblyError::DoubleObject)` when the object slot is full, ensuring no tokens are silently ignored.

## [Constraint] Empty Stem in Suffix Matching
**Learning:** `match_suffix` in `src/morphology/matcher.rs` explicitly skips matches if the resulting stem is empty. This prevents spurious matches (e.g., word "o" matching suffix "-o"), but means words that coincide with their suffix (e.g., article "o") must be handled by the lexicon or explicit checks, not morphological rules.
**Action:** When implementing morphology rules, assume `match_suffix` requires `word.len() > suffix.len()`. Add explicit test cases for this constraint to document it as intended behavior.
