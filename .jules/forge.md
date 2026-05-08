**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactoring `parse_match_pattern` Duplication**
**Learning:** Found an anti-pattern in `src/semantic/control_flow.rs` where identical logic for parsing wildcard ("αλλο"), numerals, and variables from a Greek word was fully duplicated in two branches of `parse_match_pattern` (one for raw `Word` inputs and one unwrapping `Word` from a `Phrase`).
**Action:** Extracted the 30+ line shared logic into a single `parse_word_pattern` helper function to enforce DRY principles without altering behavior.
