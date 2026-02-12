**Refactoring `analyze_argument_expr_recursive`**
**Learning:** Breaking down a large `match` statement into small, named helper functions significantly improves readability and makes the code more self-documenting. It also allows for easier testing of individual components if needed in the future.
**Action:** Always look for opportunities to extract logic from large `match` arms, especially when they contain nested logic or error handling. Use meaningful names for helper functions that describe the variant being handled (e.g., `analyze_array`, `analyze_block`).

**[Refactoring `main.rs` into `cli.rs` and `repl.rs`]**
**Learning:** Extracting CLI commands and REPL logic into separate modules (`cli.rs`, `repl.rs`) dramatically simplifies the entry point (`main.rs`) and improves separation of concerns. It makes `main.rs` purely about argument parsing and dispatching.
**Action:** When `main.rs` grows beyond basic argument parsing, extract business logic into dedicated modules immediately to prevent "God Object" tendencies in the binary entry point.
