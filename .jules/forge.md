**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactoring parse_match_pattern in control_flow.rs**
**Learning:** `parse_match_pattern` duplicated the same word parsing logic twice (once for Phrase(Word), once for Word).
**Action:** Extract the Word first with match, then apply the parsing logic once to DRY it up.
