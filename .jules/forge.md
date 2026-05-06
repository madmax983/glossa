**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored God Functions in main and disambiguation**
**Learning:** Found that the massive `cli.command` match in `main.rs` and the enormous `test_analyze_article_all_forms` test were bloating files over readability limits.
**Action:** Extracted `cli.command` processing to `execute_command(command: Commands)` and extracted the loop logic in tests to an `assert_article_context` helper, grouping test forms logically. Used `let _ = input` to silence unread variables behind features.
