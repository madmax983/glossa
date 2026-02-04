## [Vulnerability]
**Issue:** `Assembler` token limit bypass via literals
**Description:** Methods like `feed_string`, `feed_number` did not increment `token_count` or check `MAX_TOKENS`, allowing infinite accumulation of literals.
**Defense:** Updated all `feed_*` methods to enforce `MAX_TOKENS` and return `Result`. Added `tests/havoc_assembler_bypass.rs` regression test.
