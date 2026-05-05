# Refactor `test_analyze_article_all_forms` in `src/morphology/disambiguation.rs`

## Problem (Smell)
The test `test_analyze_article_all_forms` is 239 lines long. It's a "God Test" that iterates over 30+ test cases using a large hardcoded vector of tuples (`&str, Option<Case>, Option<Number>, Option<Gender>`). This triggers `clippy::too_many_lines`.

## Solution
Break this large monolithic test into smaller, logically grouped test functions:
1. `test_analyze_article_masculine`
2. `test_analyze_article_feminine`
3. `test_analyze_article_neuter`
4. `test_analyze_article_invalid`

Create a small helper macro or function to assert the expectations so we stay DRY without having one massive function.
Wait, the list of tuples is straightforward, so we can just use a helper function:
```rust
fn assert_article(
    word: &str,
    expected_case: Option<Case>,
    expected_number: Option<Number>,
    expected_gender: Option<Gender>,
) {
    let ctx_opt = analyze_article(word);
    // ...
}
```

This improves readability by splitting the assertions by gender.

## Verification
- Run `cargo test` to ensure tests still pass.
- Run `cargo clippy` to ensure warnings are resolved.
- This is strictly a test refactoring, zero runtime behavior change.
