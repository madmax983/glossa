**Refactoring `analyze_argument_expr_recursive`**
**Learning:** Breaking down a large `match` statement into small, named helper functions significantly improves readability and makes the code more self-documenting. It also allows for easier testing of individual components if needed in the future.
**Action:** Always look for opportunities to extract logic from large `match` arms, especially when they contain nested logic or error handling. Use meaningful names for helper functions that describe the variant being handled (e.g., `analyze_array`, `analyze_block`).
