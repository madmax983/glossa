//! Pattern detection for high-level language constructs
//!
//! This module acts as a "bridge" between the low-level, word-order-independent
//! Greek grammar and high-level programming patterns.
//!
//! While the [`Assembler`](crate::semantic::Assembler) handles the grammatical roles (Subject, Object, Verb),
//! this module identifies idiomatic combinations of these roles that map to complex semantics.
//!
//! # Supported Patterns
//!
//! 1.  **Struct Instantiation**: Creating new instances of user-defined types.
//! 2.  **Trait Method Calls**: Invoking methods defined on traits.
//! 3.  **Iterator Chains**: Functional programming pipelines using participles.
//!
//! # The Hero's Journey: The Iterator Chain
//!
//! Consider the task: *"Take a list of numbers, double them, keep only those greater than 10, and print them."*
//!
//! In Rust:
//! ```rust,ignore
//! vec.iter()
//!    .map(|x| x * 2)
//!    .filter(|x| x > 10)
//!    .for_each(|x| println!("{}", x));
//! ```
//!
//! In ΓΛΩΣΣΑ, we use **Participles** for mapping and **Adjectives** for filtering:
//!
//! ```glossa
//! // "xi" (the list)
//! // "doubling" (present participle -> .map())
//! // "ten greater" (comparative adjective -> .filter())
//! // "say" (imperative verb -> print)
//! ξ διπλασιαζόμενα δέκα μείζονα λέγε.
//! ```
//!
//! This module detects this sequence of grammatical constituents and transforms it
//! into the corresponding Rust iterator chain.

use super::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, AssembledStatement, CaptureMode, GlossaType,
    Scope,
};
use crate::ast::{Expr, Statement};
use crate::errors::GlossaError;
use crate::text::normalize_greek;
use smol_str::SmolStr;

/// Try to parse a trait method call: `method_name receiver`
///
/// This looks for a specific phrase pattern where a method name is immediately
/// followed by its receiver (the object it acts upon).
///
/// # Pattern
/// `[method_name] [receiver]`
///
/// # Example
/// ```text
/// // Calls the 'speak' method on the 'animal' variable
/// λέγε ζῷον.
/// ```
///
/// Returns `Some(analyzed_statement)` if this is a trait method call, `None` otherwise.
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

                        return Ok(Some(AnalyzedStatement::Expression(vec![method_call])));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Try to parse a struct instantiation
///
/// This handles the creation of new struct instances or built-in collections.
///
/// # Pattern
/// `[variable] νέον [TypeName] [arg1] [arg2] ... ἔστω`
///
/// # Example
/// ```text
/// // Define struct: εἶδος Point { x: i64, y: i64 }
///
/// // Instantiate: let p = Point { x: 10, y: 20 };
/// π νέον Σημεῖον δέκα εἴκοσι ἔστω.
/// ```
///
/// Returns `Some(analyzed_statement)` if this is a struct instantiation, `None` otherwise.
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

                return Ok(Some(AnalyzedStatement::Binding {
                    name: var_name.clone(),
                    value: collection_new,
                    mutable: true,
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

                return Ok(Some(AnalyzedStatement::Binding {
                    name: var_name.clone(),
                    value: struct_inst,
                    mutable: false,
                }));
            }
        }
    }

    Ok(None)
}

/// Detect iterator patterns with participles
///
/// This function analyzes the `AssembledStatement` to detect functional programming chains
/// expressed through Greek participles.
///
/// # Logic
/// * **Collection**: The subject or an array literal.
/// * **Map**: Present Participles (e.g., `διπλασιαζόμενα`) become `.map()`.
/// * **Filter**: Comparative Adjectives (e.g., `μείζονα`) become `.filter()`.
/// * **Fold**: The participle `συλλεγόμενα` (gathering) becomes `.fold()`.
/// * **Find**: The verb `εὑρέ` (find) becomes `.find()`.
///
/// # Example
///
/// ```text
/// // [1, 2, 3].iter().map(|x| x*2).filter(|x| x > 5).collect::<Vec<_>>()
/// [1, 2, 3] διπλασιαζόμενα πέντε μείζονα λέγε.
/// ```
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

    // Get the collection - prefer array literals, then subject
    let collection_expr = match extract_collection(asm_stmt) {
        Some(c) => c,
        None => return Ok(None),
    };

    // Determine flags for any/all quantification
    let flags = QuantifierFlags::from(asm_stmt);

    // Start with .iter()
    // chain: collection.iter()
    let mut current_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::MethodCall {
            receiver: Box::new(collection_expr),
            method: "iter".into(),
            args: vec![],
        },
        glossa_type: GlossaType::Unknown, // Iterator type
    };

    // Keep track if we performed any operations, to avoid collecting empty iterator if not needed
    let mut has_ops = false;

    // Process adjectives (filter, any, all)
    if process_adjectives(asm_stmt, &flags, &mut current_expr) {
        has_ops = true;
    }

    // Process participles (map, fold)
    let (participle_ops, is_fold_result) = process_participles(asm_stmt, &mut current_expr);
    if participle_ops {
        has_ops = true;
    }

    // Handle any/all operations with explicit operators (comparatives stored as operators)
    let is_any_all = process_explicit_quantifiers(asm_stmt, &flags, &mut current_expr);
    if is_any_all {
        return Ok(Some(current_expr));
    }

    // Handle find operations
    if is_find {
        process_find(asm_stmt, &mut current_expr);
        // Find returns Option/Result, not an iterator, so we are done
        // We assume .find() was called on current_expr
        // But wait, find returns Option<T>, so we might want to unwrap or treat as Number?
        // Current logic says: glossa_type: GlossaType::Number
        // Let's set the type properly
        current_expr.glossa_type = GlossaType::Number;
        return Ok(Some(current_expr));
    }

    // If we have a fold, we are done
    if is_fold_result {
        return Ok(Some(current_expr));
    }

    // If we have any/all result from adjectives, we are done
    // Check if the current expression type is Boolean (any/all returns bool)
    if matches!(current_expr.glossa_type, GlossaType::Boolean) {
        return Ok(Some(current_expr));
    }

    // Finalize the iterator chain (add .collect() if needed)
    // Only if we actually added operations
    if has_ops {
        finalize_iterator_chain(&mut current_expr);
        Ok(Some(current_expr))
    } else {
        Ok(None)
    }
}

// -------------------------------------------------------------------------------------------------
// Helper functions for detect_iterator_pattern
// -------------------------------------------------------------------------------------------------

/// Helper: Extract the collection expression from the assembled statement
fn extract_collection(asm_stmt: &AssembledStatement) -> Option<AnalyzedExpr> {
    if !asm_stmt.arrays.is_empty() {
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

        Some(AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(array_elements),
            glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
        })
    } else if let Some(subject) = &asm_stmt.subject {
        // Use subject only if it's not a quantifier (τι/πάντα)
        let collection_name = normalize_greek(&subject.lemma);
        if !crate::morphology::lexicon::is_any_quantifier(&collection_name)
            && !crate::morphology::lexicon::is_all_quantifier(&collection_name)
        {
            Some(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(collection_name),
                glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
            })
        } else {
            None
        }
    } else {
        None
    }
}

/// Flags to track quantifier presence (any/all)
struct QuantifierFlags {
    is_any: bool,
    is_all: bool,
}

impl QuantifierFlags {
    fn from(asm_stmt: &AssembledStatement) -> Self {
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

        Self { is_any, is_all }
    }
}

/// Helper: Process adjectives for filter/any/all patterns
/// Returns true if any operation was added
fn process_adjectives(
    asm_stmt: &AssembledStatement,
    flags: &QuantifierFlags,
    current_expr: &mut AnalyzedExpr,
) -> bool {
    // Check for comparative adjective filter/any/all pattern
    // Pattern: collection + number + comparative_adj → filter/any/all
    if asm_stmt.adjectives.is_empty() {
        return false;
    }

    let mut added = false;

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

            // Determine which method to call based on quantifier
            let method = if flags.is_any {
                "any"
            } else if flags.is_all {
                "all"
            } else {
                "filter"
            };

            // Wrap current expr
            let new_expr = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(current_expr.clone()),
                    method: method.into(),
                    args: vec![filter_closure],
                },
                glossa_type: if flags.is_any || flags.is_all {
                    GlossaType::Boolean
                } else {
                    GlossaType::Unknown
                },
            };
            *current_expr = new_expr;
            added = true;
        }
    }
    added
}

/// Helper: Process participles for map/fold patterns
/// Returns (bool, bool) -> (added_ops, is_fold_result)
fn process_participles(
    asm_stmt: &AssembledStatement,
    current_expr: &mut AnalyzedExpr,
) -> (bool, bool) {
    let mut added = false;
    let mut is_fold = false;

    for participle in &asm_stmt.participles {
        let verb_stem = normalize_greek(&participle.verb_lemma);

        // Check for fold pattern: συλλεγόμενα εἰς [target]
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
                        crate::morphology::Tense::Aorist => CaptureMode::Move,
                        // Iterator closures always take arguments, so Memoize is unsafe.
                        crate::morphology::Tense::Perfect => CaptureMode::Borrow,
                        _ => CaptureMode::Borrow,
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

                    let new_expr = AnalyzedExpr {
                        expr: AnalyzedExprKind::MethodCall {
                            receiver: Box::new(current_expr.clone()),
                            method: "fold".into(),
                            args: vec![init_expr, fold_closure],
                        },
                        glossa_type: GlossaType::Number,
                    };
                    *current_expr = new_expr;

                    is_fold = true;
                    added = true;
                    break; // Exit operators loop
                }
            }
        }

        // Skip other participle processing if this was a fold
        if is_fold {
            continue;
        }

        // Map Middle and Passive participles to .map()
        // Passive voice ("being written") is semantically valid for transformation chains
        if participle.voice == crate::morphology::Voice::Middle
            || participle.voice == crate::morphology::Voice::Passive
        {
            // Create a simple lambda based on the verb
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
                            expr: AnalyzedExprKind::NumberLiteral(2),
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

            let capture_mode = match participle.tense {
                crate::morphology::Tense::Aorist => CaptureMode::Move,
                // Iterator closures always take arguments, so Memoize is unsafe.
                // Downgrade Perfect tense to Borrow for these operations.
                crate::morphology::Tense::Perfect => CaptureMode::Borrow,
                _ => CaptureMode::Borrow,
            };

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

            let new_expr = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(current_expr.clone()),
                    method: "map".into(),
                    args: vec![closure],
                },
                glossa_type: GlossaType::Unknown,
            };
            *current_expr = new_expr;
            added = true;
        }
    }
    (added, is_fold)
}

/// Helper: Process explicit quantifier operators (any/all via >/<)
/// Returns true if an any/all operation was added
fn process_explicit_quantifiers(
    asm_stmt: &AssembledStatement,
    flags: &QuantifierFlags,
    current_expr: &mut AnalyzedExpr,
) -> bool {
    if (!flags.is_any && !flags.is_all) || asm_stmt.operators.is_empty() {
        return false;
    }

    // Get the comparison operator (Gt or Lt)
    for &bin_op in &asm_stmt.operators {
        if matches!(
            bin_op,
            crate::morphology::lexicon::BinaryOp::Gt | crate::morphology::lexicon::BinaryOp::Lt
        ) {
            let comparison_expr = extract_comparison_value(asm_stmt);
            let any_all_closure = create_comparison_predicate(bin_op, comparison_expr);

            let method = if flags.is_any { "any" } else { "all" };

            let new_expr = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(current_expr.clone()),
                    method: method.into(),
                    args: vec![any_all_closure],
                },
                glossa_type: GlossaType::Boolean,
            };
            *current_expr = new_expr;
            return true;
        }
    }
    false
}

/// Helper: Process find operations
fn process_find(asm_stmt: &AssembledStatement, current_expr: &mut AnalyzedExpr) {
    // Find operation: .iter().find(predicate)
    let mut found_predicate = false;

    if !asm_stmt.operators.is_empty() {
        // Get the comparison operator (Gt or Lt)
        for &bin_op in &asm_stmt.operators {
            if matches!(
                bin_op,
                crate::morphology::lexicon::BinaryOp::Gt | crate::morphology::lexicon::BinaryOp::Lt
            ) {
                let comparison_expr = extract_comparison_value(asm_stmt);
                let find_closure = create_comparison_predicate(bin_op, comparison_expr);

                let new_expr = AnalyzedExpr {
                    expr: AnalyzedExprKind::MethodCall {
                        receiver: Box::new(current_expr.clone()),
                        method: "find".into(),
                        args: vec![find_closure],
                    },
                    glossa_type: GlossaType::Number,
                };
                *current_expr = new_expr;
                found_predicate = true;
                break;
            }
        }
    }

    // If no predicate was added, just find the first element (essentially .next())
    if !found_predicate {
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
                capture_mode: CaptureMode::Borrow,
            },
            glossa_type: GlossaType::Function {
                params: vec![GlossaType::Number],
                returns: Box::new(GlossaType::Boolean),
            },
        };

        let new_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::MethodCall {
                receiver: Box::new(current_expr.clone()),
                method: "find".into(),
                args: vec![find_first_closure],
            },
            glossa_type: GlossaType::Number,
        };
        *current_expr = new_expr;
    }
}

/// Helper: Finalize iterator chain with optional collect
fn finalize_iterator_chain(current_expr: &mut AnalyzedExpr) {
    // Add .collect() at the end
    let new_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::MethodCall {
            receiver: Box::new(current_expr.clone()),
            method: "collect".into(),
            args: vec![],
        },
        glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
    };
    *current_expr = new_expr;
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
            crate::semantic::assembled::Literal::Number(n) => *n,
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
            capture_mode: CaptureMode::Borrow,
        },
        glossa_type: GlossaType::Function {
            params: vec![GlossaType::Number],
            returns: Box::new(GlossaType::Boolean),
        },
    }
}
