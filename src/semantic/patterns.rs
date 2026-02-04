//! Pattern detection (struct instantiation, trait calls, iterators)

use super::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedIteratorOp, AnalyzedStatement, AssembledStatement,
    GlossaType, Scope, StatementKind,
};
use crate::ast::{Expr, Statement};
use crate::errors::GlossaError;
use crate::text::normalize_greek;
use smol_str::SmolStr;

/// Try to parse a trait method call: method_name receiver
/// Returns Some(analyzed_statement) if this is a trait method call, None otherwise
pub fn try_parse_trait_method_call(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Only process Regular statements
    if let Statement::Regular { clauses, .. } = stmt {
        // Should have exactly one clause with one expression
        if clauses.len() != 1 || clauses[0].expressions.len() != 1 {
            return Ok(None);
        }

        // Should be a Phrase with exactly 2 words
        if let Expr::Phrase(terms) = &clauses[0].expressions[0] {
            if terms.len() != 2 {
                return Ok(None);
            }

            // Extract words
            if let (Expr::Word(method_word), Expr::Word(receiver_word)) = (&terms[0], &terms[1]) {
                let method_name = &method_word.normalized;
                let receiver_name = &receiver_word.normalized;

                // Check if receiver is a variable in scope
                if let Some(receiver_type) = scope.lookup(receiver_name)
                    && let GlossaType::Struct {
                        name: type_name, ..
                    } = receiver_type
                {
                    // Check if this type has a trait method with this name
                    if scope.has_trait_method(type_name, method_name) {
                        let receiver = AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(receiver_name.clone()),
                            glossa_type: receiver_type.clone(),
                        };

                        let method_call = AnalyzedExpr {
                            expr: AnalyzedExprKind::MethodCall {
                                receiver: Box::new(receiver),
                                method: method_name.clone(),
                                args: vec![],
                            },
                            glossa_type: GlossaType::Unit,
                        };

                        return Ok(Some(AnalyzedStatement {
                            kind: StatementKind::Expression,
                            expressions: vec![method_call],
                        }));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Try to parse a struct instantiation: variable νέον type_name args ἔστω
/// Returns Some(analyzed_statement) if this is a struct instantiation, None otherwise
///
/// # Pattern
///
/// `[variable] νέον [TypeName] [arg1] [arg2] ... ἔστω`
///
/// # Example
///
/// ```text
/// ξ νέον Ἀριθμός πέντε ἔστω.
/// ```
///
/// This translates to: `let xi = Number { value: 5 };`
pub fn try_parse_struct_instantiation(
    stmt: &Statement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    // Only process Regular statements
    if let Statement::Regular { clauses, .. } = stmt {
        // Should have exactly one clause
        if clauses.len() != 1 {
            return Ok(None);
        }

        let clause = &clauses[0];
        if clause.expressions.len() != 1 {
            return Ok(None);
        }

        // Should be a Phrase with at least 4 terms
        if let Expr::Phrase(terms) = &clause.expressions[0] {
            if terms.len() < 4 {
                return Ok(None);
            }

            // Verify structural words (0, 1, 2, Last) are Words
            let var_word = if let Expr::Word(w) = &terms[0] {
                w
            } else {
                return Ok(None);
            };

            let adj_word = if let Expr::Word(w) = &terms[1] {
                w
            } else {
                return Ok(None);
            };

            let type_word = if let Expr::Word(w) = &terms[2] {
                w
            } else {
                return Ok(None);
            };

            let last_word = if let Expr::Word(w) = terms.last().unwrap() {
                w
            } else {
                return Ok(None);
            };

            // Check pattern: var_name νέον TypeName args... ἔστω
            // Last word should be ἔστω (binding verb)
            if !crate::morphology::lexicon::is_binding_verb(&last_word.normalized) {
                return Ok(None);
            }

            // Second word should be νέον (new) - check both normalized form and if it's "new" via morphology
            let normalized_adj = crate::text::normalize_greek(&adj_word.normalized);
            // Check if it's "new" - could be νέον, νεον, etc.
            if normalized_adj != "νεον" && normalized_adj != "νεος" {
                return Ok(None);
            }

            // Extract components
            let var_name = &var_word.normalized;
            let type_name = &type_word.normalized;

            // Check for built-in collection types first (HashSet, HashMap)
            if let Some((rust_type, glossa_type)) =
                crate::semantic::types::detect_collection_type(type_name)
            {
                let collection_new = AnalyzedExpr {
                    expr: AnalyzedExprKind::CollectionNew {
                        collection_type: rust_type.to_string(),
                    },
                    glossa_type: glossa_type.clone(),
                };

                // Register variable in scope (collections are implicitly mutable for insert)
                scope.define_mut(var_name.clone(), glossa_type.clone());

                return Ok(Some(AnalyzedStatement {
                    kind: StatementKind::Binding {
                        name: var_name.clone(),
                        value_type: glossa_type.clone(),
                        mutable: true,
                    },
                    expressions: vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(var_name.clone()),
                            glossa_type: glossa_type.clone(),
                        },
                        collection_new,
                    ],
                }));
            }

            // Check if type exists as a user-defined struct
            if let Some(struct_type) = scope.lookup_type(type_name).cloned() {
                // Extract fields from struct type
                let fields_info: Vec<(SmolStr, GlossaType)> =
                    if let GlossaType::Struct { fields, .. } = &struct_type {
                        fields.clone()
                    } else {
                        vec![]
                    };

                let field_names: Vec<SmolStr> =
                    fields_info.iter().map(|(n, _)| n.clone()).collect();

                // Collect constructor arguments (everything between type_name and ἔστω)
                let mut args = Vec::new();
                for (i, term) in terms[3..terms.len() - 1].iter().enumerate() {
                    let expected_type = if i < fields_info.len() {
                        &fields_info[i].1
                    } else {
                        &GlossaType::Unknown
                    };

                    let analyzed_arg = match term {
                        Expr::Word(word) => {
                            // Convert word to analyzed expression
                            if let Ok(num) = word.original.parse::<i64>() {
                                // Direct numeric literal like "5" stored as word
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::NumberLiteral(num),
                                    glossa_type: GlossaType::Number,
                                }
                            } else if let Some(num) =
                                crate::morphology::lexicon::numeral_value(&word.normalized)
                            {
                                // Greek numeral word like πέντε -> 5
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::NumberLiteral(num),
                                    glossa_type: GlossaType::Number,
                                }
                            } else {
                                // Variable reference
                                let var_type = scope
                                    .lookup(&word.normalized)
                                    .cloned()
                                    .unwrap_or(GlossaType::Unknown);
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::Variable(word.normalized.clone()),
                                    glossa_type: var_type,
                                }
                            }
                        }
                        Expr::StringLiteral(s) => {
                            let lit_expr = AnalyzedExpr {
                                expr: AnalyzedExprKind::StringLiteral(s.clone()),
                                glossa_type: GlossaType::String,
                            };

                            if matches!(expected_type, GlossaType::String) {
                                // Wrap in .to_string() for struct fields expecting String
                                AnalyzedExpr {
                                    expr: AnalyzedExprKind::MethodCall {
                                        receiver: Box::new(lit_expr),
                                        method: "to_string".into(),
                                        args: vec![],
                                    },
                                    glossa_type: GlossaType::String,
                                }
                            } else {
                                lit_expr
                            }
                        }
                        Expr::NumberLiteral(n) => AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(*n),
                            glossa_type: GlossaType::Number,
                        },
                        Expr::BooleanLiteral(b) => AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(*b),
                            glossa_type: GlossaType::Boolean,
                        },
                        _ => return Ok(None), // Unsupported argument type
                    };
                    args.push(analyzed_arg);
                }

                // Build struct instantiation
                let struct_inst = AnalyzedExpr {
                    expr: AnalyzedExprKind::StructInstantiation {
                        type_name: type_name.clone(),
                        fields: field_names,
                        args,
                    },
                    glossa_type: struct_type.clone(),
                };

                // Register variable in scope with correct type
                scope.define(var_name.clone(), struct_type.clone());

                return Ok(Some(AnalyzedStatement {
                    kind: StatementKind::Binding {
                        name: var_name.clone(),
                        value_type: struct_type.clone(),
                        mutable: false,
                    },
                    expressions: vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(var_name.clone()),
                            glossa_type: struct_type.clone(),
                        },
                        struct_inst,
                    ],
                }));
            }
        }
    }

    Ok(None)
}

/// Detect iterator patterns with participles
///
/// # Pattern
/// `collection` + `participle(s)` + `verb`
///
/// # Example
/// `ξ διπλασιαζόμενα λέγε`
///
/// Translates to: `xi.iter().map(|x| x * 2).collect::<Vec<_>>()` (which is then printed)
///
/// This function analyzes the `AssembledStatement` to detect functional programming chains
/// expressed through Greek participles.
///
/// * **Present Participle (Middle)**: Maps to `.map()` (e.g., `διπλασιαζόμενα` -> doubling).
/// * **Comparative Adjective**: Maps to `.filter()` (e.g., `πέντε μείζονα` -> greater than 5).
/// * **Quantifiers (τι/πάντα)**: Maps to `.any()` or `.all()`.
/// * **Find Verb (εὑρέ)**: Maps to `.find()`.
pub fn detect_iterator_pattern(
    asm_stmt: &AssembledStatement,
    _scope: &mut Scope,
) -> Result<Option<AnalyzedExpr>, GlossaError> {
    // Need: (subject OR array) + (participles OR comparatives) + (print OR find verb)
    let verb = match &asm_stmt.verb {
        Some(v) => v,
        None => return Ok(None),
    };

    // Check if verb is a print or find verb
    let verb_lemma = normalize_greek(&verb.lemma);
    let is_print = crate::morphology::lexicon::is_print_verb(&verb_lemma);
    let is_find = crate::morphology::lexicon::is_find_verb(&verb_lemma);

    if !is_print && !is_find {
        return Ok(None);
    }

    // Get the collection - prefer array literals, then subject (but not quantifiers)
    let collection_expr = if !asm_stmt.arrays.is_empty() {
        // Use the first array literal
        let array_elements: Vec<AnalyzedExpr> = asm_stmt.arrays[0]
            .iter()
            .map(|e| match e {
                crate::ast::Expr::NumberLiteral(n) => AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(*n),
                    glossa_type: GlossaType::Number,
                },
                _ => AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(0),
                    glossa_type: GlossaType::Number,
                },
            })
            .collect();

        AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(array_elements),
            glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
        }
    } else if let Some(subject) = &asm_stmt.subject {
        // Use subject only if it's not a quantifier (τι/πάντα)
        let collection_name = normalize_greek(&subject.lemma);
        if !crate::morphology::lexicon::is_any_quantifier(&collection_name)
            && !crate::morphology::lexicon::is_all_quantifier(&collection_name)
        {
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(collection_name),
                glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
            }
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    // Start with the collection variable
    let mut iterator_ops = vec![AnalyzedIteratorOp::Iter];

    // Check for any/all quantifiers
    let mut is_any = false;
    let mut is_all = false;
    if let Some(ref subj) = asm_stmt.subject {
        let subj_lemma = normalize_greek(&subj.lemma);
        is_any = crate::morphology::lexicon::is_any_quantifier(&subj_lemma);
        is_all = crate::morphology::lexicon::is_all_quantifier(&subj_lemma);
    }
    // Also check nominatives for quantifiers
    for nom in &asm_stmt.nominatives {
        let nom_lemma = normalize_greek(&nom.lemma);
        if crate::morphology::lexicon::is_any_quantifier(&nom_lemma) {
            is_any = true;
        }
        if crate::morphology::lexicon::is_all_quantifier(&nom_lemma) {
            is_all = true;
        }
    }

    // Check for comparative adjective filter/any/all pattern
    // Pattern: collection + number + comparative_adj → filter/any/all
    // Pattern: collection + predicate_adj (implicit zero) → filter/any/all
    // Example: [1, 10, 3, 8] πέντε μείζονα → filter(|x| x > 5)
    // Example: [1, 2, 3] πάντα θετικά → all(|x| x > 0)
    if !asm_stmt.adjectives.is_empty() {
        for adj in &asm_stmt.adjectives {
            // Look up adjective in lexicon to check if it's comparative
            // Use the ORIGINAL form, not the lemma, because comparatives are irregular
            if let Some(entry) = crate::morphology::lexicon::lookup(&normalize_greek(&adj.original))
                && entry.pos == crate::morphology::PartOfSpeech::Adjective
                && let Some(rust_op) = entry.rust_equiv
                && (rust_op == ">" || rust_op == "<")
            {
                // Found a comparative adjective!
                let comparison_expr = extract_comparison_value(asm_stmt);

                // Determine the binary operation
                let bin_op = if rust_op == ">" {
                    crate::morphology::lexicon::BinaryOp::Gt
                } else {
                    crate::morphology::lexicon::BinaryOp::Lt
                };

                // Create the filter predicate: |x| x > value
                let filter_closure = create_comparison_predicate(bin_op, comparison_expr);

                // Determine which operation to use based on quantifier
                if is_any {
                    iterator_ops.push(AnalyzedIteratorOp::Any(Box::new(filter_closure.clone())));
                } else if is_all {
                    iterator_ops.push(AnalyzedIteratorOp::All(Box::new(filter_closure.clone())));
                } else {
                    iterator_ops.push(AnalyzedIteratorOp::Filter(Box::new(filter_closure.clone())));
                }
            }
        }
    }

    // Process each participle and add appropriate iterator operation
    for participle in &asm_stmt.participles {
        let verb_stem = normalize_greek(&participle.verb_lemma);

        // Check for fold pattern: συλλεγόμενα εἰς [target]
        // Pattern: collection + συλλεγόμενα + εἰς + operator(sum/product) + verb
        // Note: ἄθροισμα and γινόμενον are stored as operators, not nouns
        let mut is_fold = false;
        if verb_stem.contains("συλλεγ") {
            // Look for target operator (Add for sum, Mul for product)
            for &bin_op in &asm_stmt.operators {
                if matches!(
                    bin_op,
                    crate::morphology::lexicon::BinaryOp::Add
                        | crate::morphology::lexicon::BinaryOp::Mul
                ) {
                    // Determine initial value based on operation
                    let init_value = match bin_op {
                        crate::morphology::lexicon::BinaryOp::Add => 0,
                        crate::morphology::lexicon::BinaryOp::Mul => 1,
                        _ => unreachable!(),
                    };

                    // Determine capture mode based on participle tense
                    let capture_mode = match participle.tense {
                        crate::morphology::Tense::Aorist => crate::ast::CaptureMode::Move,
                        crate::morphology::Tense::Perfect => crate::ast::CaptureMode::Memoize,
                        _ => crate::ast::CaptureMode::Borrow,
                    };

                    // Create fold closure: |acc, x| acc + x (or acc * x)
                    let fold_body = AnalyzedExpr {
                        expr: AnalyzedExprKind::BinOp {
                            op: bin_op,
                            left: Box::new(AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable("acc".into()),
                                glossa_type: GlossaType::Number,
                            }),
                            right: Box::new(AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable("x".into()),
                                glossa_type: GlossaType::Number,
                            }),
                        },
                        glossa_type: GlossaType::Number,
                    };

                    let fold_closure = AnalyzedExpr {
                        expr: AnalyzedExprKind::Lambda {
                            params: vec!["acc".into(), "x".into()],
                            body: Box::new(fold_body),
                            capture_mode,
                        },
                        glossa_type: GlossaType::Function {
                            params: vec![GlossaType::Number, GlossaType::Number],
                            returns: Box::new(GlossaType::Number),
                        },
                    };

                    let init_expr = AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(init_value),
                        glossa_type: GlossaType::Number,
                    };

                    iterator_ops.push(AnalyzedIteratorOp::Fold {
                        init: Box::new(init_expr.clone()),
                        closure: Box::new(fold_closure.clone()),
                    });

                    is_fold = true;
                    break; // Exit operators loop
                }
            }
        }

        // Skip other participle processing if this was a fold
        if is_fold {
            continue;
        }

        // For now, map present middle participles to .map()
        // The closure will be the verb operation
        if participle.voice == crate::morphology::Voice::Middle {
            // Present middle participle: διπλασιαζόμενα → "doubling itself"
            // Maps to: .map(|x| x * 2)

            // Create a simple lambda based on the verb

            // For now, create a placeholder closure
            // In a full implementation, we'd look up the verb's operation
            let closure_body = if verb_stem.contains("διπλασιαζ") {
                // διπλασιαζω = "to double"
                AnalyzedExpr {
                    expr: AnalyzedExprKind::BinOp {
                        op: crate::morphology::lexicon::BinaryOp::Mul,
                        left: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable("x".into()),
                            glossa_type: GlossaType::Number,
                        }),
                        right: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::Literal(2),
                            glossa_type: GlossaType::Number,
                        }),
                    },
                    glossa_type: GlossaType::Number,
                }
            } else {
                // Default: just return x
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()),
                    glossa_type: GlossaType::Unknown,
                }
            };

            // Determine capture mode based on participle tense
            // Present participle: borrow (streaming operation)
            // Aorist participle: move (one-shot consumption)
            // Perfect participle: memoize (cached result)
            let capture_mode = match participle.tense {
                crate::morphology::Tense::Aorist => crate::ast::CaptureMode::Move,
                crate::morphology::Tense::Perfect => crate::ast::CaptureMode::Memoize,
                _ => crate::ast::CaptureMode::Borrow, // Present, Imperfect, etc.
            };

            // Create closure: |x| body
            let closure = AnalyzedExpr {
                expr: AnalyzedExprKind::Lambda {
                    params: vec!["x".into()],
                    body: Box::new(closure_body),
                    capture_mode,
                },
                glossa_type: GlossaType::Function {
                    params: vec![GlossaType::Number],
                    returns: Box::new(GlossaType::Number),
                },
            };

            // Lower the closure to HIR and wrap in iterator op
            iterator_ops.push(AnalyzedIteratorOp::Map(Box::new(closure.clone())));
        }
    }

    // Handle any/all operations with operators (comparatives stored as operators)
    if (is_any || is_all) && !asm_stmt.operators.is_empty() {
        // Get the comparison operator (Gt or Lt)
        for &bin_op in &asm_stmt.operators {
            if matches!(
                bin_op,
                crate::morphology::lexicon::BinaryOp::Gt | crate::morphology::lexicon::BinaryOp::Lt
            ) {
                let comparison_expr = extract_comparison_value(asm_stmt);
                let any_all_closure = create_comparison_predicate(bin_op, comparison_expr);

                if is_any {
                    iterator_ops.push(AnalyzedIteratorOp::Any(Box::new(any_all_closure.clone())));
                } else {
                    iterator_ops.push(AnalyzedIteratorOp::All(Box::new(any_all_closure.clone())));
                }

                // Build the iterator chain for any/all (returns boolean)
                let iterator_chain = AnalyzedExpr {
                    expr: AnalyzedExprKind::IteratorChain {
                        collection: Box::new(collection_expr),
                        ops: iterator_ops,
                    },
                    glossa_type: GlossaType::Boolean,
                };
                return Ok(Some(iterator_chain));
            }
        }
    }

    // Handle find operations differently from print operations
    if is_find {
        // Find operation: .iter().find(predicate)
        if !asm_stmt.operators.is_empty() {
            // Get the comparison operator (Gt or Lt)
            for &bin_op in &asm_stmt.operators {
                if matches!(
                    bin_op,
                    crate::morphology::lexicon::BinaryOp::Gt
                        | crate::morphology::lexicon::BinaryOp::Lt
                ) {
                    let comparison_expr = extract_comparison_value(asm_stmt);
                    let find_closure = create_comparison_predicate(bin_op, comparison_expr);

                    iterator_ops.push(AnalyzedIteratorOp::Find(Box::new(find_closure.clone())));
                    break;
                }
            }
        }

        // If no predicate was added, just find the first element (essentially .next())
        // Use .find(|_| true) to get the first element
        if iterator_ops.len() <= 1 {
            // No predicate specified, so find the first element
            // Create a trivial predicate that always returns true: |_| true
            let always_true_body = AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            };

            let find_first_closure = AnalyzedExpr {
                expr: AnalyzedExprKind::Lambda {
                    params: vec!["_".into()],
                    body: Box::new(always_true_body),
                    capture_mode: crate::ast::CaptureMode::Borrow,
                },
                glossa_type: GlossaType::Function {
                    params: vec![GlossaType::Number],
                    returns: Box::new(GlossaType::Boolean),
                },
            };

            iterator_ops.push(AnalyzedIteratorOp::Find(Box::new(
                find_first_closure.clone(),
            )));
        }

        // Build the iterator chain for find (no .collect())
        let iterator_chain = AnalyzedExpr {
            expr: AnalyzedExprKind::IteratorChain {
                collection: Box::new(collection_expr),
                ops: iterator_ops,
            },
            glossa_type: GlossaType::Number, // find returns Option<T>, but we'll unwrap for now
        };

        return Ok(Some(iterator_chain));
    }

    // Print operation: only proceed if we have actual operations
    if iterator_ops.len() <= 1 {
        // No filter/map operations were added, so this isn't an iterator pattern
        return Ok(None);
    }

    // Check if this is a fold/any/all operation (returns single value, not a collection)
    let has_fold = iterator_ops
        .iter()
        .any(|op| matches!(op, AnalyzedIteratorOp::Fold { .. }));
    let has_any = iterator_ops
        .iter()
        .any(|op| matches!(op, AnalyzedIteratorOp::Any(_)));
    let has_all = iterator_ops
        .iter()
        .any(|op| matches!(op, AnalyzedIteratorOp::All(_)));

    if has_fold {
        // Fold returns a single value, no .collect() needed
        let iterator_chain = AnalyzedExpr {
            expr: AnalyzedExprKind::IteratorChain {
                collection: Box::new(collection_expr),
                ops: iterator_ops,
            },
            glossa_type: GlossaType::Number, // fold returns a single number
        };
        return Ok(Some(iterator_chain));
    }

    if has_any || has_all {
        // Any/all return a boolean, no .collect() needed
        let iterator_chain = AnalyzedExpr {
            expr: AnalyzedExprKind::IteratorChain {
                collection: Box::new(collection_expr),
                ops: iterator_ops,
            },
            glossa_type: GlossaType::Boolean,
        };
        return Ok(Some(iterator_chain));
    }

    // Add .collect() at the end for map/filter operations
    iterator_ops.push(AnalyzedIteratorOp::Collect);

    // Build the iterator chain expression
    let iterator_chain = AnalyzedExpr {
        expr: AnalyzedExprKind::IteratorChain {
            collection: Box::new(collection_expr),
            ops: iterator_ops,
        },
        glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
    };

    Ok(Some(iterator_chain))
}

/// Helper: Extract comparison value for filter/find/any/all
fn extract_comparison_value(asm_stmt: &AssembledStatement) -> AnalyzedExpr {
    if let Some(genitive) = asm_stmt.genitives.first() {
        // Genitive of comparison: θου μείζονα = "greater than theta"
        // For single-letter variables, strip genitive ending
        let normalized = normalize_greek(&genitive.original);
        let var_name = if let Some(stem) = normalized.strip_suffix("ου") {
            // Strip -ου genitive ending (θου → θ)
            stem.to_string()
        } else if let Some(stem) = normalized.strip_suffix("ης") {
            // Strip -ης genitive ending
            stem.to_string()
        } else if let Some(stem) = normalized.strip_suffix("ων") {
            // Strip -ων genitive plural ending
            stem.to_string()
        } else {
            // Use as-is (shouldn't happen for valid genitives)
            normalized.to_string()
        };
        AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(var_name.into()),
            glossa_type: GlossaType::Number,
        }
    } else if let Some(literal) = asm_stmt.literals.first() {
        // Literal comparison: πέντε μείζονα = "greater than five"
        let value = match literal {
            crate::semantic::assembler::Literal::Number(n) => *n,
            _ => 0,
        };
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(value),
            glossa_type: GlossaType::Number,
        }
    } else {
        // Implicit zero: θετικά = "positive" = greater than 0
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        }
    }
}

/// Helper: Create comparison predicate closure |x| x op value
fn create_comparison_predicate(
    op: crate::morphology::lexicon::BinaryOp,
    comparison_value: AnalyzedExpr,
) -> AnalyzedExpr {
    let predicate_body = AnalyzedExpr {
        expr: AnalyzedExprKind::BinOp {
            op,
            left: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("x".into()),
                glossa_type: GlossaType::Number,
            }),
            right: Box::new(comparison_value),
        },
        glossa_type: GlossaType::Boolean,
    };

    AnalyzedExpr {
        expr: AnalyzedExprKind::Lambda {
            params: vec!["x".into()],
            body: Box::new(predicate_body),
            capture_mode: crate::ast::CaptureMode::Borrow,
        },
        glossa_type: GlossaType::Function {
            params: vec![GlossaType::Number],
            returns: Box::new(GlossaType::Boolean),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Word, Statement, Clause};
    use crate::morphology::{Case, Gender, Number, Person, Tense, Voice, Mood};
    use crate::semantic::{Constituent, VerbConstituent};

    // Helper to create a Word
    fn word(original: &str) -> Word {
        Word::new(original)
    }

    // Helper to create a Phrase expression
    fn phrase(terms: Vec<Expr>) -> Expr {
        Expr::Phrase(terms)
    }

    // Helper to create a Regular Statement
    fn regular_statement(exprs: Vec<Expr>) -> Statement {
        Statement::Regular {
            clauses: vec![Clause { expressions: exprs }],
            is_query: false,
            is_propagate: false,
        }
    }

    #[test]
    fn test_struct_instantiation_success() {
        // ξ νέον Χρήστης "Alice" 42 ἔστω.
        let stmt = regular_statement(vec![
            phrase(vec![
                Expr::Word(word("ξ")),
                Expr::Word(word("νέον")),
                Expr::Word(word("Χρήστης")),
                Expr::StringLiteral("Alice".to_string()),
                Expr::NumberLiteral(42),
                Expr::Word(word("ἔστω")),
            ])
        ]);

        let mut scope = Scope::new();
        // Define struct type in scope
        let struct_type = GlossaType::Struct {
            name: "χρηστης".into(),
            gender: Gender::Masculine,
            fields: vec![
                ("ονομα".into(), GlossaType::String),
                ("ηλικια".into(), GlossaType::Number),
            ],
        };
        scope.define_type("χρηστης", struct_type.clone());

        let result = try_parse_struct_instantiation(&stmt, &mut scope).unwrap();
        assert!(result.is_some());

        let analyzed = result.unwrap();
        // Use ref name to avoid moving name out of analyzed.kind
        assert!(matches!(analyzed.kind, StatementKind::Binding { ref name, .. } if name == "ξ"));

        // Check inferred type
        if let StatementKind::Binding { value_type, .. } = analyzed.kind {
            assert_eq!(value_type, struct_type);
        }
    }

    #[test]
    fn test_struct_instantiation_missing_keywords() {
        let mut scope = Scope::new();

        // Missing "νέον" -> ξ παλιόν Χρήστης ...
        let stmt_bad_new = regular_statement(vec![
            phrase(vec![
                Expr::Word(word("ξ")),
                Expr::Word(word("παλιόν")),
                Expr::Word(word("Χρήστης")),
                Expr::Word(word("ἔστω")),
            ])
        ]);
        assert!(try_parse_struct_instantiation(&stmt_bad_new, &mut scope).unwrap().is_none());

        // Missing "ἔστω" -> ... λέγε
        let stmt_bad_verb = regular_statement(vec![
            phrase(vec![
                Expr::Word(word("ξ")),
                Expr::Word(word("νέον")),
                Expr::Word(word("Χρήστης")),
                Expr::Word(word("λέγε")),
            ])
        ]);
        assert!(try_parse_struct_instantiation(&stmt_bad_verb, &mut scope).unwrap().is_none());
    }

    #[test]
    fn test_iterator_pattern_map() {
        // ξ διπλασιαζόμενα λέγε (Present Middle Participle -> Map)
        let asm = AssembledStatement {
            subject: Some(Constituent {
                lemma: "ξ".into(),
                original: "ξ".into(),
                case: Case::Nominative,
                number: Some(Number::Plural),
                gender: Some(Gender::Neuter),
                person: Some(Person::Third),
            }),
            verb: Some(VerbConstituent {
                lemma: "λεγω".into(),
                original: "λέγε".into(),
                person: Some(Person::Second),
                number: Some(Number::Singular),
                tense: Some(Tense::Present),
                mood: Some(Mood::Imperative),
                voice: Some(Voice::Active),
            }),
            participles: vec![
                crate::semantic::assembler::ParticipleConstituent {
                    verb_lemma: "διπλασιαζω".into(),
                    original: "διπλασιαζόμενα".into(),
                    tense: Tense::Present,
                    voice: Voice::Middle,
                    case: Case::Nominative,
                    gender: Gender::Neuter,
                    number: Number::Plural,
                }
            ],
            // Defaults for others
            nominatives: vec![],
            object: None,
            indirect: None,
            genitives: vec![],
            adjectives: vec![],
            literals: vec![],
            arrays: vec![],
            index_accesses: vec![],
            property_accesses: vec![],
            operators: vec![],
            blocks: vec![],
            nested_phrases: vec![],
            unwraps: vec![],
            has_mutable_marker: false,
            is_query: false,
            is_propagate: false,
            has_containment_preposition: false,
            has_delimiter_preposition: false,
            string_method: None,
        };

        let mut scope = Scope::new();
        // Define collection in scope
        scope.define("ξ", GlossaType::List(Box::new(GlossaType::Number)));

        let result = detect_iterator_pattern(&asm, &mut scope).unwrap();
        assert!(result.is_some());

        let analyzed = result.unwrap();
        if let AnalyzedExprKind::IteratorChain { ops, .. } = analyzed.expr {
            assert!(ops.iter().any(|op| matches!(op, AnalyzedIteratorOp::Map(_))));
            assert!(ops.iter().any(|op| matches!(op, AnalyzedIteratorOp::Collect)));
        } else {
            panic!("Expected IteratorChain, got {:?}", analyzed.expr);
        }
    }

    #[test]
    fn test_iterator_pattern_filter() {
        // ξ πέντε μείζονα λέγε (Filter > 5)
        let asm = AssembledStatement {
            subject: Some(Constituent {
                lemma: "ξ".into(),
                original: "ξ".into(),
                case: Case::Nominative,
                number: Some(Number::Plural),
                gender: Some(Gender::Neuter),
                person: Some(Person::Third),
            }),
            verb: Some(VerbConstituent {
                lemma: "λεγω".into(),
                original: "λέγε".into(),
                person: Some(Person::Second),
                number: Some(Number::Singular),
                tense: Some(Tense::Present),
                mood: Some(Mood::Imperative),
                voice: Some(Voice::Active),
            }),
            literals: vec![crate::semantic::assembler::Literal::Number(5)],
            adjectives: vec![
                Constituent {
                    lemma: "μειζων".into(), // Normalized lemma
                    original: "μείζονα".into(), // Original form for lookup
                    case: Case::Nominative,
                    number: Some(Number::Plural),
                    gender: Some(Gender::Neuter),
                    person: None,
                }
            ],
            // Defaults
            participles: vec![],
            nominatives: vec![],
            object: None,
            indirect: None,
            genitives: vec![],
            arrays: vec![],
            index_accesses: vec![],
            property_accesses: vec![],
            operators: vec![],
            blocks: vec![],
            nested_phrases: vec![],
            unwraps: vec![],
            has_mutable_marker: false,
            is_query: false,
            is_propagate: false,
            has_containment_preposition: false,
            has_delimiter_preposition: false,
            string_method: None,
        };

        let mut scope = Scope::new();
        scope.define("ξ", GlossaType::List(Box::new(GlossaType::Number)));

        let result = detect_iterator_pattern(&asm, &mut scope).unwrap();

        assert!(result.is_some(), "Expected iterator pattern to be detected for Filter");
        let analyzed = result.unwrap();
        if let AnalyzedExprKind::IteratorChain { ops, .. } = analyzed.expr {
            assert!(ops.iter().any(|op| matches!(op, AnalyzedIteratorOp::Filter(_))));
        } else {
            panic!("Expected IteratorChain, got {:?}", analyzed.expr);
        }
    }

    #[test]
    fn test_iterator_pattern_no_match() {
        let asm = AssembledStatement {
            subject: Some(Constituent {
                lemma: "ξ".into(),
                original: "ξ".into(),
                case: Case::Nominative,
                number: Some(Number::Plural),
                gender: Some(Gender::Neuter),
                person: Some(Person::Third),
            }),
            verb: Some(VerbConstituent {
                lemma: "λεγω".into(),
                original: "λέγε".into(),
                person: Some(Person::Second),
                number: Some(Number::Singular),
                tense: Some(Tense::Present),
                mood: Some(Mood::Imperative),
                voice: Some(Voice::Active),
            }),
            // No participles or adjectives
            participles: vec![],
            adjectives: vec![],
            literals: vec![],
            nominatives: vec![],
            object: None,
            indirect: None,
            genitives: vec![],
            arrays: vec![],
            index_accesses: vec![],
            property_accesses: vec![],
            operators: vec![],
            blocks: vec![],
            nested_phrases: vec![],
            unwraps: vec![],
            has_mutable_marker: false,
            is_query: false,
            is_propagate: false,
            has_containment_preposition: false,
            has_delimiter_preposition: false,
            string_method: None,
        };

        let mut scope = Scope::new();
        let result = detect_iterator_pattern(&asm, &mut scope).unwrap();
        assert!(result.is_none());
    }
}
