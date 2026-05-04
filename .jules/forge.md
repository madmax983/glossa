**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**Refactored `src/main.rs` and `test_analyze_article_all_forms`**
**Learning:** Found two "God Functions" violating the persona constraints. The `main` function was monolithic due to repeating `cfg(feature="nova")` bounds in a single match block. The `test_analyze_article_all_forms` was a monolithic test over 200 lines long, triggering warnings.
**Action:** Extracted the massive match block into `execute_command(command: Option<Commands>) -> Result<()>` which flattens the file significantly. Splitted `test_analyze_article_all_forms` logically by article gender and extracted the shared test loop logic into a helper `verify_article_cases` to satisfy DRY, avoiding massive copy-pasting.
