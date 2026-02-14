use crate::errors::GlossaError;
use crate::semantic::assembler::state::AssembledStatement;
use crate::semantic::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr,
};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
use crate::text::normalize_greek;

/// Helper: Detect property access print pattern (pi.xi)
pub fn classify_property_access_print(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_print_verb(&verb_lemma)
            && !asm_stmt.genitives.is_empty()
            && let Some(subject) = &asm_stmt.subject
        {
            // Get owner from genitive (use lemma to get base variable name)
            let owner_lemma = &asm_stmt.genitives[0].lemma;

            // Get property from subject (nominative)
            let property = normalize_greek(&subject.original);

            // Check if owner is a struct type in scope
            if let Some(owner_type) = scope.lookup(owner_lemma)
                && matches!(owner_type, GlossaType::Struct { .. })
            {
                // Build property access
                let prop_access = AnalyzedExpr {
                    expr: AnalyzedExprKind::PropertyAccess {
                        owner: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(owner_lemma.clone()),
                            glossa_type: owner_type.clone(),
                        }),
                        property: property.clone(),
                    },
                    glossa_type: GlossaType::Unknown, // TODO: Look up field type
                };

                return Ok(Some(AnalyzedStatement::Print(vec![prop_access])));
            }
        }
    }
    Ok(None)
}

/// Helper: Detect print statement
pub fn classify_print(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_print_verb(&verb_lemma) {
            // Binary expr with operator
            if !asm_stmt.operators.is_empty() {
                let left = if let Some(ref subj) = asm_stmt.subject {
                    scope.lookup(&subj.lemma).map(|var_type| AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    })
                } else {
                    None
                };

                let right = asm_stmt.literals.first().map(literal_to_analyzed_expr);

                if let (Some(left_expr), Some(right_expr)) = (left, right) {
                    let op = asm_stmt.operators[0];
                    let bin_expr = build_binary_expr(left_expr, op, right_expr);
                    return Ok(Some(AnalyzedStatement::Print(vec![bin_expr])));
                }
            }

            // Property access
            if !asm_stmt.property_accesses.is_empty() {
                let mut args = Vec::new();
                for (owner, method) in &asm_stmt.property_accesses {
                    let receiver = AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(owner.clone().into()),
                        glossa_type: scope.lookup(owner).cloned().unwrap_or(GlossaType::Unknown),
                    };
                    let method_args = if let Some((ref meth, ref delim)) = asm_stmt.string_method {
                        if meth == method {
                            vec![AnalyzedExpr {
                                expr: AnalyzedExprKind::StringLiteral(delim.clone()),
                                glossa_type: GlossaType::String,
                            }]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    };
                    let return_type = match method.as_str() {
                        "len" => GlossaType::Number,
                        "split" => GlossaType::List(Box::new(GlossaType::String)),
                        "join" => GlossaType::String,
                        _ => GlossaType::Unknown,
                    };
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::MethodCall {
                            receiver: Box::new(receiver),
                            method: method.clone().into(),
                            args: method_args,
                        },
                        glossa_type: return_type,
                    });
                }
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            // Index access
            if !asm_stmt.index_accesses.is_empty() {
                let mut args = Vec::new();
                for (array_expr, index_expr) in &asm_stmt.index_accesses {
                    let array_analyzed = analyze_argument_expr(array_expr, scope)?;
                    let index_analyzed = analyze_argument_expr(index_expr, scope)?;
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::IndexAccess {
                            array: Box::new(array_analyzed),
                            index: Box::new(index_analyzed),
                        },
                        glossa_type: GlossaType::Unknown,
                    });
                }
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            // Unwrap
            if !asm_stmt.unwraps.is_empty() {
                let mut args = Vec::new();
                for unwrap_expr in &asm_stmt.unwraps {
                    let inner_analyzed = analyze_argument_expr(unwrap_expr, scope)?;
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::Unwrap(Box::new(inner_analyzed)),
                        glossa_type: GlossaType::Unknown,
                    });
                }
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            // Default
            let mut args =
                build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators);

            if let Some(ref subj) = asm_stmt.subject
                && let Some(var_type) = scope.lookup(&subj.lemma)
            {
                args.insert(
                    0,
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    },
                );
            }

            if let Some(ref obj) = asm_stmt.object
                && let Some(var_type) = scope.lookup(&obj.lemma)
            {
                args.push(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                    glossa_type: var_type.clone(),
                });
            }

            return Ok(Some(AnalyzedStatement::Print(args)));
        }
    }
    Ok(None)
}

/// Helper: Detect query statement
pub fn classify_query(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if asm_stmt.is_query {
        // Containment pattern
        if asm_stmt.has_containment_preposition
            && let Some(ref subj) = asm_stmt.subject
        {
            let subj_name = normalize_greek(&subj.original);
            let subj_type = scope
                .lookup(&subj_name)
                .cloned()
                .unwrap_or(GlossaType::Unknown);

            let collection = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj_name.clone()),
                glossa_type: subj_type.clone(),
            };

            let element = if let Some(lit) = asm_stmt.literals.first() {
                literal_to_analyzed_expr(lit)
            } else {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(0),
                    glossa_type: GlossaType::Number,
                }
            };

            let is_map = matches!(subj_type, GlossaType::Map(_, _));
            let method = if is_map { "contains_key" } else { "contains" };

            // Handle referencing argument if not a string literal
            let arg_expr = if matches!(element.expr, AnalyzedExprKind::StringLiteral(_)) {
                element
            } else {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::UnaryOp {
                        op: crate::morphology::lexicon::UnaryOp::Ref,
                        operand: Box::new(element),
                    },
                    glossa_type: GlossaType::Unknown,
                }
            };

            let contains_expr = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(collection),
                    method: method.into(),
                    args: vec![arg_expr],
                },
                glossa_type: GlossaType::Boolean,
            };

            return Ok(Some(AnalyzedStatement::Query(vec![contains_expr])));
        }

        // Regular query
        let mut exprs = Vec::new();
        for lit in &asm_stmt.literals {
            exprs.push(literal_to_analyzed_expr(lit));
        }
        if let Some(ref subj) = asm_stmt.subject {
            let var_type = scope
                .lookup(&subj.lemma)
                .cloned()
                .unwrap_or(GlossaType::Unknown);
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type,
            });
        }
        return Ok(Some(AnalyzedStatement::Query(exprs)));
    }
    Ok(None)
}
