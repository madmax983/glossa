1. **Identify Missing Coverage Again:**
   - The code coverage dropped to exactly 85.71% again. The test I added `test_parse_return_expression_invalid` must be failing on `asm.finalize()?` and returning an Err, covering the `?` line.
   - However, the second branch `match crate::semantic::conversion::extract_value(&asm_stmt, scope)` can ALSO return an `Err(e)`. The line `Err(e) => Err(e)` is likely still uncovered!

2. **Add a Second Invalid Test:**
   - I need a test that successfully finalizes in the assembler (`finalize() -> Ok(...)`) but fails in `extract_value()`.
   - A sentence with multiple objects, or something that assembler accepts as a statement but doesn't produce a value.
   - For example: `Expr::Word(Word::new("δός")), Expr::Word(Word::new("ἄνθρωπος")), Expr::Word(Word::new("τρέχει"))`. This evaluates to "man runs", which evaluates to an assignment or standard statement.
   - If I feed this into the assembler, it finalizes as a regular statement (verb + subject), but `extract_value` on it will fail! Because `extract_value` expects a value-yielding statement (like a math operation, object pushing, etc.).
   - Wait, `extract_value` fails when it's not a value yielding statement. Let's create a clause: `δός` (Return) followed by `ἄνθρωπος τρέχει` (man runs).

3. **Verify:**
   - Add the test, run formatting, test locally with llvm-cov, and then submit.
