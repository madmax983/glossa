import re

with open("src/semantic/control_flow.rs", "r") as f:
    content = f.read()

new_func = """fn parse_match_expression(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if stmt.clauses().is_empty() {
        return Err(GlossaError::semantic(
            "Match expression needs at least one clause",
        ));
    }

    // Extract scrutinee from clause 0, expression 0 (skip κατά)
    let scrutinee_clause = &stmt.clauses()[0];

    // Instead of assembling it, let's just parse the remaining words as a pattern!
    // Since "κατὰ ξ" becomes "ξ", which is an expression, we can just call parse_match_pattern.

    let mut modified_expr = scrutinee_clause.expressions[0].clone();
    if let Expr::Phrase(terms) = &mut modified_expr {
        if !terms.is_empty() {
            terms.remove(0); // Remove "κατὰ"
        }
    }

    let scrutinee = match parse_match_pattern(&modified_expr, scope) {
        Ok(s) => s,
        Err(_) => {
            let synthetic_clause = Clause {
                expressions: vec![scrutinee_clause.expressions[0].clone()],
            };

            // `skip_first_word_and_parse` uses `assemble_statement` under the hood.
            // If `assemble_statement` panics due to `MissingVerb` here, we bypass it.
            // Wait, `skip_first_word_and_parse` has a manual fallback.

            let res = skip_first_word_and_parse(&synthetic_clause, scope);
            match res {
                Ok(r) => r,
                Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb)) => {
                     // Since `skip_first_word_and_parse` failed, let's just treat the scrutinee as unknown!
                     AnalyzedExpr {
                         expr: AnalyzedExprKind::Variable("UNKNOWN_MATCH_SCRUTINEE".into()),
                         glossa_type: crate::semantic::GlossaType::Unknown,
                     }
                }
                Err(e) => return Err(e),
            }
        }
    };

    // Build arms: pattern from clause[i], expr 1 → body from clause[i+1], expr 0
    let mut arms = Vec::with_capacity(stmt.clauses().len() / 2);

    for i in 0..stmt.clauses().len() {
        let clause = &stmt.clauses()[i];

        // Get pattern from expression 1 (if it exists)
        if clause.expressions.len() > 1 {
            let pattern_expr = &clause.expressions[1];

            // Extract pattern value
            let pattern = parse_match_pattern(pattern_expr, scope)?;

            // The body is in the next clause's first expression
            let body_stmts = if i + 1 < stmt.clauses().len() {
                let next_clause = &stmt.clauses()[i + 1];
                let body_stmt = Statement::Regular {
                    clauses: vec![next_clause.clone()],
                    is_query: false,
                    is_propagate: false,
                };

                // Using analyze_statement instead of assemble_statement directly
                let analyzed_stmts = crate::semantic::analyze_statement(&body_stmt, scope)?;
                if analyzed_stmts.is_empty() {
                    return Err(GlossaError::semantic("Empty match arm body"));
                }
                analyzed_stmts
            } else {
                return Err(GlossaError::semantic("Empty match arm body"));
            };

            arms.push((pattern, body_stmts));
        }
    }

    Ok(Some(AnalyzedStatement::Match { scrutinee, arms }))
}"""

content = re.sub(
    r'fn parse_match_expression\(\n    stmt: &Statement,\n    scope: &mut Scope,\n\) -> Result<Option<AnalyzedStatement>, GlossaError> \{.*?\n    Ok\(Some\(AnalyzedStatement::Match \{ scrutinee, arms \}\)\)\n\}',
    new_func,
    content,
    flags=re.DOTALL
)

with open("src/semantic/control_flow.rs", "w") as f:
    f.write(content)
