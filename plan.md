1. **Fix Undefined Variable Compilation Silencing:**
   - I will use `run_in_bash_session` to run a python script that modifies `src/semantic/conversion.rs`. The script will use `.find()` to dynamically extract the target strings and `.replace()` to apply the fixes.
   - I will run the following command:
```bash
cat << 'IN_EOF' > patch_conversion.py
content = open("src/semantic/conversion.rs").read()

# Fix 1: Fallback extraction
start = content.find("    // Fallback: If no literals/ops, check Subject/Object")
end = content.find("    if asm_stmt.is_propagate && !exprs.is_empty() {")
target = content[start:end]
replace = """    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            if !scope.is_defined(&subj.lemma) && !asm_stmt.is_propagate {
                return Err(GlossaError::undefined(subj.lemma.as_str()));
            }
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            if !scope.is_defined(&obj.lemma) && !asm_stmt.is_propagate {
                return Err(GlossaError::undefined(obj.lemma.as_str()));
            }
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }

"""
content = content.replace(target, replace)

# Fix 2: Print default
start = content.find("fn try_print_default(")
end = content.find("/// Helper: Detect print statement")
target = content[start:end]
replace = """fn try_print_default(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Vec<AnalyzedExpr>, GlossaError> {
    let mut args =
        build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators)?;

    if let Some(ref subj) = asm_stmt.subject {
        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
    }

    if let Some(ref obj) = asm_stmt.object {
        if let Some(var_type) = scope.lookup(&obj.lemma) {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: var_type.clone(),
            });
        } else {
            return Err(GlossaError::undefined(obj.lemma.as_str()));
        }
    }

    Ok(args)
}

"""
content = content.replace(target, replace)

# Fix 3: Query
start = content.find("    // Regular query")
end = content.find("    Ok(Some(AnalyzedStatement::Query(exprs)))")
target = content[start:end]
replace = """    // Regular query
    let mut exprs = Vec::with_capacity(asm_stmt.literals.len() + 1);
    for lit in &asm_stmt.literals {
        exprs.push(literal_to_analyzed_expr(lit));
    }
    if let Some(ref subj) = asm_stmt.subject {
        if !scope.is_defined(&subj.lemma) {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
        let var_type = scope
            .lookup(&subj.lemma)
            .cloned()
            .unwrap_or(GlossaType::Unknown);
        exprs.push(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
            glossa_type: var_type,
        });
    }
"""
content = content.replace(target, replace)

open("src/semantic/conversion.rs", "w").write(content)
IN_EOF
python3 patch_conversion.py
```

2. **Verify changes to `src/semantic/conversion.rs`:**
   - I will run `git diff src/semantic/conversion.rs` to review the applied modifications.

3. **Fix Double Subject and Missing Verb Checks:**
   - I will use `run_in_bash_session` to run a python script that modifies `src/semantic/assembly/mod.rs`. The script will use `.find()` to dynamically extract the target strings and `.replace()` to apply the fixes.
   - I will run the following command:
```bash
cat << 'IN_EOF' > patch_assembly.py
content = open("src/semantic/assembly/mod.rs").read()

# Fix 1: Double Subject missing verb case
start = content.find("        // Check subject-verb agreement if both present")
end = content.find("        // Return the assembled statement")
target = content[start:end]
replace = """        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }
        } else if self.state.subject.is_some()
            && !self.state.nominatives.is_empty()
            && self.state.operators.is_empty()
        {
            // Unhandled double subject when there's no verb and no operators
            // Exception: Function definition / pattern calls
            let is_function_call = !self.state.nested_phrases.is_empty()
                || !self.state.blocks.is_empty()
                || !self.state.literals.is_empty();
            let is_special_pattern =
                !self.state.property_accesses.is_empty() || self.state.is_query;
            if !is_function_call && !is_special_pattern {
                // No verb, stacked nominatives...
                return Err(AssemblyError::DoubleSubject);
            }
        } else if self.state.verb.is_none() && self.state.subject.is_some() && self.state.object.is_none() && !self.state.nominatives.is_empty() {
             return Err(AssemblyError::DoubleSubject);
        }
"""
content = content.replace(target, replace)

# Fix 2: Missing verb codegen panic
start = content.find("        if ctx.is_match_arm")
end = content.find("    /// Check for special markers")
target = content[start:end]
replace = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
        {
            // Only skip missing verb for match arms if they actually have content like adjectives or literals
            if self.state.adjectives.is_empty() && self.state.literals.is_empty() && self.state.operators.is_empty() && self.state.unwraps.is_empty() && self.state.blocks.is_empty() && self.state.nested_phrases.is_empty() && self.state.arrays.is_empty() {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }
        Err(AssemblyError::MissingVerb)
    }
"""
content = content.replace(target, replace)

open("src/semantic/assembly/mod.rs", "w").write(content)
IN_EOF
python3 patch_assembly.py
```

4. **Verify changes to `src/semantic/assembly/mod.rs`:**
   - I will run `git diff src/semantic/assembly/mod.rs` to review the applied modifications.

5. **Verify the fixes through Tests:**
   - I will run `cargo test` to ensure all functionality is preserved.
   - I will use `run_in_bash_session` to run:
```bash
cat << 'IN_EOF' > test_undef.gl
ἄγνωστος λέγε.
IN_EOF
cargo run -- run test_undef.gl
```
   - I will use `run_in_bash_session` to run:
```bash
cat << 'IN_EOF' > test_double.gl
ὁ ἄνθρωπος ὁ θεὸς λέγει.
IN_EOF
cargo run -- run test_double.gl
```
   - I will use `run_in_bash_session` to run:
```bash
cat << 'IN_EOF' > test_missing_verb.gl
ὁ ἄνθρωπος.
IN_EOF
cargo run -- run test_missing_verb.gl
```

6. **Clean up artifacts:**
   - I will use `run_in_bash_session` to run `rm test_undef.gl test_double.gl test_missing_verb.gl patch_conversion.py patch_assembly.py` to remove temporary files.

7. **Complete pre-commit steps:**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
