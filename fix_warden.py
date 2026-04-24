import re

with open("src/semantic/control_flow.rs", "r") as f:
    content = f.read()

# restore skip_first_word_and_parse
new_func = """fn skip_first_word_and_parse(
    clause: &Clause,
    scope: &mut Scope,
) -> Result<AnalyzedExpr, GlossaError> {
    // Create a modified clause without the first word
    let mut modified_clause = clause.clone();

    // Remove the first word from the first expression
    if let Some(first_expr) = modified_clause.expressions.first_mut()
        && let Expr::Phrase(terms) = first_expr
        && !terms.is_empty()
    {
        terms.remove(0);

        // Return exactly one variable instead of assembling
        if modified_clause.expressions.len() == 1 {
            if let Expr::Phrase(terms) = &modified_clause.expressions[0] {
                if terms.len() == 1 {
                    let lemma = crate::morphology::lexicon::get_lemma(&terms[0]);

                    if let Some(val) = crate::morphology::lexicon::numeral_value(&lemma) {
                         return Ok(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(val),
                            glossa_type: crate::semantic::GlossaType::Number,
                        });
                    }

                    if lemma == "αλλο" {
                         return Ok(AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(true),
                            glossa_type: crate::semantic::GlossaType::Boolean,
                        });
                    }

                    if lemma == "μηδεν" || lemma == "ουδεν" || lemma == "τι" || lemma == "εν" {
                         return Ok(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(lemma.into()),
                            glossa_type: crate::semantic::GlossaType::Unknown,
                        });
                    }

                    let glossa_type = scope.lookup(&lemma).cloned().unwrap_or(crate::semantic::GlossaType::Unknown);
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(lemma.into()),
                        glossa_type,
                    });
                }
            }
        }
    }

    // Parse the modified clause as a statement
    let stmt = Statement::Regular {
        clauses: vec![modified_clause],
        is_query: false,
        is_propagate: false,
    };

    // We bypass the top-level MissingVerb check by extracting the expression manually
    // from the assembled statement if it's a simple fallback condition.
    let analyzed_res = assemble_statement(&stmt);
    let analyzed = match analyzed_res {
        Ok(a) => a,
        Err(crate::errors::AssemblyError::MissingVerb) => {
            return Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb));
        }
        Err(e) => return Err(GlossaError::AssemblyError(e)),
    };

    let converted =
        match crate::semantic::conversion::convert_assembled_to_analyzed(&analyzed, scope) {
            Ok(c) => c,
            Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb)) => {
                if let Some(ref subj) = analyzed.subject {
                    let var_expr = crate::semantic::AnalyzedExpr {
                        expr: crate::semantic::AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: crate::semantic::GlossaType::Unknown,
                    };
                    AnalyzedStatement::Expression(vec![var_expr])
                } else if let Some(ref obj) = analyzed.object {
                    let var_expr = crate::semantic::AnalyzedExpr {
                        expr: crate::semantic::AnalyzedExprKind::Variable(obj.lemma.clone()),
                        glossa_type: crate::semantic::GlossaType::Unknown,
                    };
                    AnalyzedStatement::Expression(vec![var_expr])
                } else {
                    return Err(GlossaError::semantic("Empty condition in conditional"));
                }
            }
            Err(e) => return Err(e),
        };

    match converted {
        AnalyzedStatement::Expression(exprs) => {
            if exprs.is_empty() {
                Err(GlossaError::semantic("Empty expression in clause"))
            } else {
                Ok(exprs[0].clone())
            }
        }
        _ => Err(GlossaError::semantic("Expected expression, found statement")),
    }
}"""

content = re.sub(
    r'fn skip_first_word_and_parse\(\n    clause: &Clause,\n    scope: &mut Scope,\n\) -> Result<AnalyzedExpr, GlossaError> \{.*?\n        _ => Err\(GlossaError::semantic\("Expected expression, found statement"\)\),\n    \}\n\}',
    new_func,
    content,
    flags=re.DOTALL
)

with open("src/semantic/control_flow.rs", "w") as f:
    f.write(content)
