**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Extract Tester Presentation Formatting**
**Learning:** Tools handling output visualization often grow into 'God Functions' combining multiple formatting steps (e.g. summary, detail table, failure extraction).
**Action:** Extract generic presentation logic (like rendering tables) into specialized helper functions to strictly isolate UI concerns and keep the main function flat.
