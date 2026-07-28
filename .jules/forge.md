**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored parse_match_pattern**
**Learning:** Found exact duplicated code handling wildcard, numeral, and variable resolution in both Phrase and Word match arms of `parse_match_pattern`.
**Action:** Extracted the logic into a named helper function `analyze_match_word`.
