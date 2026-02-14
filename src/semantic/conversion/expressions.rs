use crate::errors::GlossaError;
use crate::semantic::assembler::state::{AssembledStatement, Constituent, Literal};
use crate::semantic::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr, literal_to_type,
};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
use crate::text::normalize_greek;

/// Helper: Default expression
pub fn classify_expression(
    asm_stmt: &AssembledStatement,
) -> Result<AnalyzedStatement, GlossaError> {
    let mut exprs =
        build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators);

    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }

    if asm_stmt.is_propagate && !exprs.is_empty() {
        let last_expr = exprs.pop().unwrap();
        let try_expr = AnalyzedExpr {
            glossa_type: last_expr.glossa_type.clone(),
            expr: AnalyzedExprKind::Try(Box::new(last_expr)),
        };
        exprs.push(try_expr);
    }

    Ok(AnalyzedStatement::Expression(exprs))
}

/// Helper: Common logic for genitive method call parsing
#[allow(clippy::collapsible_if)]
pub fn try_parse_genitive_method_call(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Option<(AnalyzedExpr, GlossaType)> {
    if let Some(ref subject) = asm_stmt.subject {
        if !asm_stmt.genitives.is_empty() {
            let owner_lemma = &asm_stmt.genitives[0].lemma;
            let method_name = normalize_greek(&subject.original);

            if let Some(owner_type) = scope.lookup(owner_lemma) {
                if !scope.is_defined(&method_name) {
                    let receiver = AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(owner_lemma.clone()),
                        glossa_type: owner_type.clone(),
                    };

                    let mut args = Vec::new();
                    for lit in &asm_stmt.literals {
                        args.push(literal_to_analyzed_expr(lit));
                    }

                    return Some((
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::MethodCall {
                                receiver: Box::new(receiver),
                                method: method_name,
                                args,
                            },
                            glossa_type: GlossaType::Unknown,
                        },
                        GlossaType::Unknown,
                    ));
                }
            }
        }
    }
    None
}

/// Helper: Detect Enum variant (None, Some, Ok, Err)
fn detect_enum_variant(
    word: &Constituent,
    literals: &[Literal],
) -> Option<(AnalyzedExpr, GlossaType)> {
    let lemma = normalize_greek(&word.lemma);
    let original = normalize_greek(&word.original);

    // None
    if crate::morphology::lexicon::is_none_word(&lemma)
        || crate::morphology::lexicon::is_none_word(&original)
    {
        return Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: GlossaType::Option(Box::new(GlossaType::Unknown)),
            },
            GlossaType::Option(Box::new(GlossaType::Unknown)),
        ));
    }

    // Some
    if (crate::morphology::lexicon::is_some_word(&lemma)
        || crate::morphology::lexicon::is_some_word(&original))
        && let Some(lit) = literals.first()
    {
        let inner_expr = literal_to_analyzed_expr(lit);
        let inner_type = inner_expr.glossa_type.clone();
        return Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Some(Box::new(inner_expr)),
                glossa_type: GlossaType::Option(Box::new(inner_type.clone())),
            },
            GlossaType::Option(Box::new(inner_type)),
        ));
    }

    // Ok
    if (crate::morphology::lexicon::is_ok_word(&lemma)
        || crate::morphology::lexicon::is_ok_word(&original))
        && let Some(lit) = literals.first()
    {
        let inner_expr = literal_to_analyzed_expr(lit);
        let inner_type = inner_expr.glossa_type.clone();
        return Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Ok(Box::new(inner_expr)),
                glossa_type: GlossaType::Result(
                    Box::new(inner_type.clone()),
                    Box::new(GlossaType::String),
                ),
            },
            GlossaType::Result(Box::new(inner_type), Box::new(GlossaType::String)),
        ));
    }

    // Err
    if (crate::morphology::lexicon::is_err_word(&lemma)
        || crate::morphology::lexicon::is_err_word(&original))
        && let Some(lit) = literals.first()
    {
        let inner_expr = literal_to_analyzed_expr(lit);
        let inner_type = inner_expr.glossa_type.clone();
        return Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Err(Box::new(inner_expr)),
                glossa_type: GlossaType::Result(
                    Box::new(GlossaType::Unknown),
                    Box::new(inner_type.clone()),
                ),
            },
            GlossaType::Result(Box::new(GlossaType::Unknown), Box::new(inner_type)),
        ));
    }

    None
}

/// Extract value from assembled statement
pub fn extract_value(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<(AnalyzedExpr, GlossaType), GlossaError> {
    // Check for unwrap expressions first
    if !asm_stmt.unwraps.is_empty() {
        let inner_analyzed = analyze_argument_expr(&asm_stmt.unwraps[0], scope)?;
        return Ok((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(inner_analyzed)),
                glossa_type: GlossaType::Unknown, // Type will be inferred
            },
            GlossaType::Unknown,
        ));
    }

    // Check subject for Option/Result words
    if let Some(ref subj) = asm_stmt.subject
        && let Some(result) = detect_enum_variant(subj, &asm_stmt.literals)
    {
        return Ok(result);
    }

    // Check for genitive method call (Subject of Genitive)
    if let Some(result) = try_parse_genitive_method_call(asm_stmt, scope) {
        return Ok(result);
    }

    // Check nominatives for Option/Result words
    for nom in &asm_stmt.nominatives {
        if let Some(result) = detect_enum_variant(nom, &asm_stmt.literals) {
            return Ok(result);
        }
    }

    // If we have property accesses, use the first one
    if let Some((owner, method)) = asm_stmt.property_accesses.first() {
        let receiver = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(owner.clone().into()),
            glossa_type: GlossaType::Unknown,
        };
        return Ok((
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: method.clone().into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Number,
            },
            GlossaType::Number,
        ));
    }

    // If we have index accesses, use the first one
    if let Some((array_expr, index_expr)) = asm_stmt.index_accesses.first() {
        let array_analyzed = analyze_argument_expr(array_expr, scope)?;
        let index_analyzed = analyze_argument_expr(index_expr, scope)?;
        return Ok((
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(array_analyzed),
                    index: Box::new(index_analyzed),
                },
                glossa_type: GlossaType::Unknown, // Element type is unknown without inference
            },
            GlossaType::Unknown,
        ));
    }

    // If we have arrays, use the first array
    if let Some(array_elements) = asm_stmt.arrays.first() {
        let mut analyzed_elements = Vec::with_capacity(array_elements.len());
        for e in array_elements {
            analyzed_elements.push(analyze_argument_expr(e, scope)?);
        }

        let element_type = analyzed_elements
            .first()
            .map(|e| e.glossa_type.clone())
            .unwrap_or(GlossaType::Unknown);
        return Ok((
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(analyzed_elements),
                glossa_type: GlossaType::List(Box::new(element_type)),
            },
            GlossaType::List(Box::new(GlossaType::Unknown)),
        ));
    }

    // If we have operators, build a binary expression
    if !asm_stmt.operators.is_empty() {
        // Check if we can build from literals alone (2+ literals)
        if asm_stmt.literals.len() >= 2 {
            let exprs =
                build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators);
            if let Some(expr) = exprs.into_iter().next() {
                let ty = expr.glossa_type.clone();
                return Ok((expr, ty));
            }
        }

        // Or check if we can combine object + literal with operator
        if let Some(ref obj) = asm_stmt.object
            && !asm_stmt.literals.is_empty()
        {
            // Build: object op literal
            let left = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown, // Will be inferred
            };
            let right = literal_to_analyzed_expr(&asm_stmt.literals[0]);
            let op = asm_stmt.operators[0];
            let bin_expr = build_binary_expr(left, op, right);
            let ty = bin_expr.glossa_type.clone();
            return Ok((bin_expr, ty));
        }
    }

    // Prefer literals (single value, no operators)
    if let Some(lit) = asm_stmt.literals.first() {
        return Ok((literal_to_analyzed_expr(lit), literal_to_type(lit)));
    }

    // Otherwise use object
    if let Some(ref obj) = asm_stmt.object {
        // Check both lemma and original form
        let obj_lemma = normalize_greek(&obj.lemma);
        let obj_original = normalize_greek(&obj.original);

        // Check for None (οὐδέν)
        if crate::morphology::lexicon::is_none_word(&obj_lemma)
            || crate::morphology::lexicon::is_none_word(&obj_original)
        {
            return Ok((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::None,
                    glossa_type: GlossaType::Option(Box::new(GlossaType::Unknown)),
                },
                GlossaType::Option(Box::new(GlossaType::Unknown)),
            ));
        }

        // Check for Some (τί) with a value
        if crate::morphology::lexicon::is_some_word(&obj_lemma)
            || crate::morphology::lexicon::is_some_word(&obj_original)
        {
            // Get the inner value from literals
            if let Some(lit) = asm_stmt.literals.first() {
                let inner_expr = literal_to_analyzed_expr(lit);
                let inner_type = inner_expr.glossa_type.clone();
                return Ok((
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Some(Box::new(inner_expr)),
                        glossa_type: GlossaType::Option(Box::new(inner_type.clone())),
                    },
                    GlossaType::Option(Box::new(inner_type)),
                ));
            }
        }

        // Check for Ok (ἐπιτυχία) with a value
        if crate::morphology::lexicon::is_ok_word(&obj_lemma)
            || crate::morphology::lexicon::is_ok_word(&obj_original)
        {
            // Get the inner value from literals
            if let Some(lit) = asm_stmt.literals.first() {
                let inner_expr = literal_to_analyzed_expr(lit);
                let inner_type = inner_expr.glossa_type.clone();
                // LIMITATION: Error type defaults to String. See nominatives section for explanation.
                return Ok((
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Ok(Box::new(inner_expr)),
                        glossa_type: GlossaType::Result(
                            Box::new(inner_type.clone()),
                            Box::new(GlossaType::String),
                        ),
                    },
                    GlossaType::Result(Box::new(inner_type), Box::new(GlossaType::String)),
                ));
            }
        }

        // Check for Err (σφάλμα) with a value
        if crate::morphology::lexicon::is_err_word(&obj_lemma)
            || crate::morphology::lexicon::is_err_word(&obj_original)
        {
            // Get the error value from literals
            if let Some(lit) = asm_stmt.literals.first() {
                let inner_expr = literal_to_analyzed_expr(lit);
                let inner_type = inner_expr.glossa_type.clone();
                // LIMITATION: Success type defaults to Unknown. See nominatives section for explanation.
                return Ok((
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Err(Box::new(inner_expr)),
                        glossa_type: GlossaType::Result(
                            Box::new(GlossaType::Unknown),
                            Box::new(inner_type.clone()),
                        ),
                    },
                    GlossaType::Result(Box::new(GlossaType::Unknown), Box::new(inner_type)),
                ));
            }
        }

        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(&obj_lemma) {
            return Ok((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            ));
        }

        return Ok((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            },
            GlossaType::Unknown,
        ));
    }

    // Default
    Ok((
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
        GlossaType::Number,
    ))
}
