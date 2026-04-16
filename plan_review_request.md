Oh! Wait.
If `is_equals_verb` is true, it evaluates it to `AnalyzedExprKind::AssertEq`!
Wait, `AssertEq` is NOT `BinOp(==)`!
`AssertEq` is a statement for testing: `δοκιμή`.
Wait, in Ancient Greek `ἐστὶ` means "is". `ἴσον` means "equal".
But `εως ξ εστι ισον 5` should NOT be parsed as `AssertEq`!
In `src/semantic/conversion.rs`:
Wait! `classify_equality_assertion` specifically generates `AnalyzedExprKind::AssertEq` which evaluates to `GlossaType::Unit`.
Why did `test_parse_while_loop_success` evaluate to `Number` then?
Because `condition.glossa_type` in the loop evaluated to `Number`!
Wait! Why didn't `classify_equality_assertion` process it?
Because `scope.lookup(&subj.lemma)`! `subj.lemma` is `"ξ"`. Does scope have `"ξ"`? Yes, the test predefined it.
But wait! `classify_equality_assertion` only applies if `is_equals_verb` is true! Is "ειμι" an equals verb?
Let's check `is_equals_verb("ειμι")`!
No! "ἰσοῦται" (is equal to) is the equality verb!
Ah! "ειμι" is "to be". It's not an equals verb!
So `classify_equality_assertion` is bypassed completely because `verb_lemma` is "ειμι", NOT "ἰσοῦται"!

Wait, if it's not an `AssertEq`, where does `BinOp(==)` get generated?
`build_expressions_from_literals_and_ops`!
Or `classify_expression` fallback!
```rust
    let (literals_to_build, operators_to_build) = if !asm_stmt.operators.is_empty()
        && asm_stmt.literals.len() < asm_stmt.operators.len() + 1
    {
        // Fallback case: operators likely depend on Subject/Object
        (asm_stmt.literals.as_slice(), &[][..])
    }
```
Does `Assembler` identify `ισον` as an operator?!
Let's check `lexicon::is_operator`.
Yes! `ισος`, `ισον` is an equality operator `==` in `lexicon::is_equality_adjective` and `is_operator`.
Wait, if it's an operator, then `asm_stmt.operators` is NOT EMPTY! It contains `Eq`.
So `Assembler` extracts `Eq` into `asm_stmt.operators`!
Let's check `Assembler::feed_expr`.
If it's `is_operator`, it pushes to `operators`.

Then in `classify_expression`:
```rust
    let actively_attempts_action = asm_stmt.verb.is_some() || !asm_stmt.literals.is_empty();
    let has_binary_fallback = !asm_stmt.operators.is_empty() && asm_stmt.literals.len() < 2;

    if actively_attempts_action
        && asm_stmt.subject.is_some()
        && !asm_stmt.nominatives.is_empty()
        && !has_binary_fallback
    {
        return Err(GlossaError::AssemblyError(DoubleSubject));
    }
```
Wait, if it's evaluated by `classify_expression`, it will generate `BinOp(==)`!
BUT `classify_query` intercepts it before `classify_expression` is reached!!
Because I set `is_query = true`!
Ah!!! If I set `is_query = true`, `classify_query` will run BEFORE `classify_expression` and intercept it!
And `classify_query` doesn't know about operators! It just dumps everything into a `Query` list!
And then `skip_first_word_and_parse` extracts the first element of that list (`NumberLiteral(5)`).

So the problem is that `is_query = true` forces the statement to be processed by `classify_query`, which bypasses `classify_expression`!
If I shouldn't use `is_query = true`, how do I bypass the `MissingVerb` check?
Wait, if it has `is_operator_expr = !self.state.operators.is_empty()`, then `MissingVerb` is bypassed anyway!
Wait! `test_match_basic` is `κατὰ ξ`.
`κατὰ ξ` is a preposition and a variable. It has NO operators, NO literals.
So it is NOT an operator expression! It is just `subject` = `ξ`.
So `Assembler::finalize()` throws `MissingVerb` for `κατὰ ξ`!
But wait! If I just temporarily set `is_query = true` during `assemble_statement(&stmt)`, that works! But THEN I should set it BACK to `false` before `convert_assembled_to_analyzed`!
```rust
    let mut analyzed = assemble_statement(&stmt)?;
    analyzed.is_query = false; // Reset it!
    let converted = crate::semantic::conversion::convert_assembled_to_analyzed(&analyzed, scope)?;
```
If I set `is_query = false` on `analyzed` BEFORE passing to `convert_assembled_to_analyzed`, then `classify_query` won't intercept it, and it will correctly fall through to `classify_expression`!
YES!!!
This perfectly aligns with the memory: "Safely bypass AssemblyError::MissingVerb errors during their evaluation in src/semantic/control_flow.rs by temporarily setting the synthetic statement's is_query flag to true ... and then unpacking the resulting AnalyzedStatement::Query back into an AnalyzedStatement::Expression."
Wait! The memory says "unpacking the resulting AnalyzedStatement::Query back into an AnalyzedStatement::Expression".
This means the memory intended for `classify_query` to actually intercept it, and then we unpack the `Query`!
BUT if `classify_query` intercepts it, we lose the operators!
Actually, if I just `analyzed.is_query = false` before conversion, I don't need to unpack `AnalyzedStatement::Query` because it will just evaluate to an `Expression` naturally!
Wait, but if I do `analyzed.is_query = false`, does `Assembler::finalize()` return an `AssembledStatement` that we can modify? Yes, `analyzed` is mutable.
Let's see if `AssembledStatement` has `is_query` public. Yes, all fields are public (or accessible in crate).
