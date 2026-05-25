1.  **Fix Undefined Variable Silently Becoming 0:**
    *   In `src/semantic/conversion.rs`, the `extract_value` method falls back to a `NumberLiteral(0)` and `GlossaType::Number` when it can't find a value in an expression. We need to update this fallback behavior.
    *   Currently, the fallback looks like this:
        ```rust
        // Default
        Ok((
            AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(0),
                glossa_type: GlossaType::Number,
            },
            GlossaType::Number,
        ))
        ```
    *   This logic should either be replaced, or we should explicitly ensure that an undefined variable triggers an `UndefinedName` error before it reaches this fallback. We will modify `extract_value` to catch undefined references appropriately and trigger `GlossaError::undefined(...)` rather than returning the `0` literal.

2.  **Fix Double Subject Silently Compiling:**
    *   In `src/semantic/assembly/mod.rs`, the `finalize` method attempts to catch double subjects.
    *   Currently, there is a check:
        ```rust
        if !is_function_call && !is_special_pattern {
            // No verb, stacked nominatives...
            return Err(AssemblyError::DoubleSubject);
        }
        ```
    *   However, if there is a verb (which `ὁ ἄνθρωπος ὁ θεὸς λέγει.` has, `λέγει`), it might bypass the double subject check if the check doesn't properly fire when multiple nominatives are present alongside a verb. We'll update `finalize` to accurately detect and reject statements with multiple subjects (nominatives) that shouldn't have them, returning `AssemblyError::DoubleSubject`.

3.  **Fix Missing Verb `CodegenError` to Use `MissingVerb` Proper Error:**
    *   The user gets an ICE (Internal Compiler Error) because `missing_verb` check returns `AssemblyError::MissingVerb` directly from assembly, but somewhere this might be caught or ignored, or maybe it generates an invalid AST that causes `codegen` to panic or fail later.
    *   Looking at `check_missing_verb` in `src/semantic/assembly/mod.rs`:
        ```rust
        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if subject.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }
        Err(AssemblyError::MissingVerb)
        ```
    *   This logic is very hardcoded `subject.lemma == "ανθρωπος"`. We will fix this logic to properly enforce that statements need a verb unless they fall under specific exemptions (like assignments, properties, literals).

4.  **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
