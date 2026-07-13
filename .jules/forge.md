**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.
**Refactored CLI main function**
**Learning:** Found a god object function `main()` in `src/main.rs` over 200 lines long, filled with deeply nested `match` statements and repetitive error-handling boilerplate.
**Action:** Flattened the execution path by moving all commands out to an `execute_command` helper function, using guard clauses to remove nesting, and extracting the boilerplate error generation to a helper.
