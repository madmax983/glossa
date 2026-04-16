Oh! The `MissingVerb` occurs during `compile_to_rust(source)`!
Wait, `test_match_basic` uses `ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.`.
Why does this throw `MissingVerb`?
Because `κατὰ ξ` is a clause! `μηδὲν ᾖ` is a clause!
Wait! The grammar for `clause_list` says `clause ~ (comma ~ clause)*` and a `clause` is `expression ~ (chain ~ expression)*`.
So `κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε`
Wait! The statement is `κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.`
This is ONE statement separated by `,` and `·`!
When `analyze_statement` processes it, it calls `analyze_control_flow`.
`analyze_control_flow` identifies `κατὰ` as a match particle. It calls `parse_match_expression`.
```rust
fn parse_match_expression(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // ...
    let scrutinee_clause = &stmt.clauses()[0];
    let synthetic_clause = Clause {
        expressions: vec![scrutinee_clause.expressions[0].clone()],
    };
    let scrutinee = skip_first_word_and_parse(&synthetic_clause, scope)?;
```
`synthetic_clause` is `κατὰ ξ`.
`skip_first_word_and_parse` skips `κατὰ` and parses `ξ`.
`ξ` is just a variable!
So `skip_first_word_and_parse` calls `assemble_statement(&stmt)?` on `ξ`!
`assemble_statement` fails with `MissingVerb` because it's a verbless construct and we made `MissingVerb` strict!
But wait! `skip_first_word_and_parse` already has a fallback for `MissingVerb`:
```rust
    let analyzed = assemble_statement(&stmt)?;

    // We bypass the top-level MissingVerb check by extracting the expression manually
    // from the assembled statement if it's a simple fallback condition.
    let converted =
        match crate::semantic::conversion::convert_assembled_to_analyzed(&analyzed, scope) {
            Ok(c) => c,
            Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb)) => {
                // If the conversion failed ONLY because of a MissingVerb check (which was added for top-level
                // statements to prevent Codegen ICEs), we can still extract the variable manually.
                // Control flow conditions are allowed to be verbless expressions (e.g. `κατὰ ξ`).
                if let Some(ref subj) = analyzed.subject {
                    let var_expr = crate::semantic::AnalyzedExpr {
                        expr: crate::semantic::AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: crate::semantic::GlossaType::Unknown,
                    };
                    AnalyzedStatement::Expression(vec![var_expr])
                } else if let Some(ref obj) = analyzed.object { ... }
```
Wait! `assemble_statement(&stmt)?` is what throws `MissingVerb` BEFORE we even get to `convert_assembled_to_analyzed`!
Ah! `assemble_statement` returns `Result<AssembledStatement, GlossaError>` and it uses `?`.
So `MissingVerb` is returned from `assemble_statement(&stmt)?` directly and bubbles up!
Yes! `assemble_statement` calls `asm.finalize()?`!
```rust
pub fn assemble_statement(stmt: &Statement) -> Result<AssembledStatement, GlossaError> {
    // ...
    Ok(asm.finalize()?)
}
```
If `asm.finalize()` returns `MissingVerb`, `assemble_statement` returns `Err(GlossaError::AssemblyError(MissingVerb))`!
And `skip_first_word_and_parse` uses `?` on `assemble_statement`, so it immediately returns the error, skipping the `match` fallback block!

```rust
    let analyzed = assemble_statement(&stmt)?; // <--- HERE IT BAILS OUT EARLY
```

To fix this, the code should be:
```rust
    let analyzed = match assemble_statement(&stmt) {
        Ok(a) => a,
        Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb)) => {
            // Re-assemble but with is_query=true to bypass MissingVerb
            let mut stmt_query = stmt.clone(); // Wait, stmt is not cloneable?
        }
    };
```
But the memory says:
"Safely bypass `AssemblyError::MissingVerb` errors during their evaluation in `src/semantic/control_flow.rs` by temporarily setting the synthetic statement's `is_query` flag to `true` (as queries are exempt from verb requirements), and then unpacking the resulting `AnalyzedStatement::Query` back into an `AnalyzedStatement::Expression`."

AHA! Memory already told me EXACTLY how to fix this, I just didn't see it because it wasn't mentioned in the Echo issue!
"temporarily setting the synthetic statement's `is_query` flag to `true` (as queries are exempt from verb requirements), and then unpacking the resulting `AnalyzedStatement::Query` back into an `AnalyzedStatement::Expression`."

So I need to modify `skip_first_word_and_parse` to set `is_query = true`.
Let's see `skip_first_word_and_parse` in `control_flow.rs`:
