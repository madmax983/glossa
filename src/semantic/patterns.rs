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
    let Statement::Regular { clauses, .. } = stmt else {
        return Ok(None);
    };

    // Should have exactly one clause
    if clauses.len() != 1 {
        return Ok(None);
    }

    let clause = &clauses[0];
    if clause.expressions.len() != 1 {
        return Ok(None);
    }

    // Should be a Phrase with at least 4 terms
    let Expr::Phrase(terms) = &clause.expressions[0] else {
        return Ok(None);
    };

    if terms.len() < 4 {
        return Ok(None);
    }

    // Verify structural words (0, 1, 2, Last) are Words
    let Expr::Word(var_word) = &terms[0] else {
        return Ok(None);
    };
    let Expr::Word(adj_word) = &terms[1] else {
        return Ok(None);
    };
    let Expr::Word(type_word) = &terms[2] else {
        return Ok(None);
    };
    let Some(Expr::Word(last_word)) = terms.last() else {
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
    let Some(struct_type) = scope.lookup_type(type_name).cloned() else {
        // If it looks like a struct instantiation (var νέον Type ... ἔστω)
        // but the type is unknown, return an error instead of falling back.
        return Err(GlossaError::undefined(type_name.to_string()));
    };

    // Extract fields from struct type
    let fields_info: Vec<(SmolStr, GlossaType)> =
        if let GlossaType::Struct { fields, .. } = &struct_type {
            fields.clone()
        } else {
            vec![]
        };

    let field_names: Vec<SmolStr> = fields_info.iter().map(|(n, _)| n.clone()).collect();

    // Collect constructor arguments (everything between type_name and ἔστω)
    let Some(args) = parse_struct_args(&terms[3..terms.len() - 1], &fields_info, scope) else {
        return Ok(None);
    };

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

    Ok(Some(AnalyzedStatement::Binding {
        name: var_name.clone(),
        value: struct_inst,
        mutable: false,
    }))
}

fn parse_struct_args(
    args_terms: &[Expr],
    fields_info: &[(SmolStr, GlossaType)],
    scope: &Scope,
) -> Option<Vec<AnalyzedExpr>> {
    let mut args = Vec::with_capacity(args_terms.len());
    for (i, term) in args_terms.iter().enumerate() {
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
            _ => return None, // Unsupported argument type
        };
        args.push(analyzed_arg);
    }
    Some(args)
}

/// Detect iterator patterns with participles
///
/// This function analyzes the `AssembledStatement` to detect functional programming chains
/// expressed through Greek participles.
///
/// # Linguistic Logic
///
/// In Ancient Greek, participles function as verbal adjectives. They modify the noun
/// by describing an action it is performing.
///
/// * **Transformation**: A participle like "doubling" (διπλασιαζόμενα) describes the numbers
///   *as they are being doubled*. This maps naturally to `.map()`.
/// * **Filtering**: A comparative adjective like "greater" (μείζονα) limits the scope
///   of the noun. This maps to `.filter()`.
/// * **Reduction**: The verb "gather" (συλλέγω) implies bringing things together.
///   The participle "gathering" (συλλεγόμενα) maps to `.fold()`.
///
/// # Mapping Table
///
/// | Greek Construct | Morphological Feature | Rust Equivalent |
/// |----------------|-----------------------|-----------------|
/// | **Participle** | Middle/Passive Voice | `.map(closure)` |
/// | **Participle** | "Gathering" lemma | `.fold(init, closure)` |
/// | **Adjective** | Comparative (`>`) | `.filter(predicate)` |
/// | **Quantifier** | "Any" (`τι`) + `>` | `.any(predicate)` |
/// | **Quantifier** | "All" (`πάντα`) + `>` | `.all(predicate)` |
/// | **Verb** | "Find" (`εὑρέ`) | `.find(predicate)` |
pub fn detect_iterator_pattern(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
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
    let (is_any, is_all) = get_quantifiers(asm_stmt);

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
    if process_adjectives(asm_stmt, scope, is_any, is_all, &mut current_expr) {
        has_ops = true;
    }

    // Process participles (map, fold)
    let (participle_ops, is_fold_result) = process_participles(asm_stmt, &mut current_expr);
    if participle_ops {
        has_ops = true;
    }

    // Handle any/all operations with explicit operators (comparatives stored as operators)
    let is_any_all =
        process_explicit_quantifiers(asm_stmt, scope, is_any, is_all, &mut current_expr);
    if is_any_all {
        return Ok(Some(current_expr));
    }

    // Handle find operations
    if is_find {
        process_find(asm_stmt, scope, &mut current_expr);
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
fn get_quantifiers(asm_stmt: &AssembledStatement) -> (bool, bool) {
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

    (is_any, is_all)
}

/// Helper: Process adjectives for filter/any/all patterns
/// Returns true if any operation was added
fn process_adjectives(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
    is_any: bool,
    is_all: bool,
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
            let comparison_expr = extract_comparison_value(asm_stmt, scope);

            // Determine the binary operation
            let bin_op = if rust_op == ">" {
                crate::morphology::lexicon::BinaryOp::Gt
            } else {
                crate::morphology::lexicon::BinaryOp::Lt
            };

            // Create the filter predicate: |x| x > value
            let filter_closure = create_comparison_predicate(bin_op, comparison_expr);

            // Determine which method to call based on quantifier
            let method = if is_any {
                "any"
            } else if is_all {
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
                glossa_type: if is_any || is_all {
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
    scope: &Scope,
    is_any: bool,
    is_all: bool,
    current_expr: &mut AnalyzedExpr,
) -> bool {
    if (!is_any && !is_all) || asm_stmt.operators.is_empty() {
        return false;
    }

    // Get the comparison operator (Gt or Lt)
    for &bin_op in &asm_stmt.operators {
        if matches!(
            bin_op,
            crate::morphology::lexicon::BinaryOp::Gt | crate::morphology::lexicon::BinaryOp::Lt
        ) {
            let comparison_expr = extract_comparison_value(asm_stmt, scope);
            let any_all_closure = create_comparison_predicate(bin_op, comparison_expr);

            let method = if is_any { "any" } else { "all" };

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
fn process_find(asm_stmt: &AssembledStatement, scope: &Scope, current_expr: &mut AnalyzedExpr) {
    // Find operation: .iter().find(predicate)
    let mut found_predicate = false;

    if !asm_stmt.operators.is_empty() {
        // Get the comparison operator (Gt or Lt)
        for &bin_op in &asm_stmt.operators {
            if matches!(
                bin_op,
                crate::morphology::lexicon::BinaryOp::Gt | crate::morphology::lexicon::BinaryOp::Lt
            ) {
                let comparison_expr = extract_comparison_value(asm_stmt, scope);
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
fn extract_comparison_value(asm_stmt: &AssembledStatement, scope: &Scope) -> AnalyzedExpr {
    if let Some(genitive) = asm_stmt.genitives.first() {
        // Strategy 1: Use the lemma from morphological analysis (handles known irregulars like ὀνόματος -> ὄνομα)
        let lemma = normalize_greek(&genitive.lemma);
        if scope.is_defined(&lemma) {
            return AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(lemma),
                glossa_type: GlossaType::Number,
            };
        }

        // Strategy 2: Fallback to manual stripping for unknown words (handles implicit variables)
        let normalized = normalize_greek(&genitive.original);
        let stripped_name = if let Some(stem) = normalized.strip_suffix("ου") {
            // Strip -ου genitive ending (θου → θ)
            stem.to_string()
        } else if let Some(stem) = normalized.strip_suffix("ης") {
            // Strip -ης genitive ending
            stem.to_string()
        } else if let Some(stem) = normalized.strip_suffix("ων") {
            // Strip -ων genitive plural ending
            stem.to_string()
        } else {
            // Use as-is
            normalized.to_string()
        };

        // If stripped name exists, use it
        if scope.is_defined(&stripped_name) {
            return AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(stripped_name.into()),
                glossa_type: GlossaType::Number,
            };
        }

        // Strategy 3: Check if original name exists (maybe variable IS the genitive form?)
        if scope.is_defined(&normalized) {
            return AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(normalized),
                glossa_type: GlossaType::Number,
            };
        }

        // Default: use stripped name if we found one, or lemma as fallback
        // This maintains backward compatibility and provides reasonable error messages
        let final_name = if stripped_name != normalized {
            stripped_name
        } else {
            lemma.to_string()
        };

        AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(final_name.into()),
            glossa_type: GlossaType::Number,
        }
    } else if let Some(literal) = asm_stmt.literals.first() {
        // Literal comparison: πέντε μείζονα = "greater than five"
        let value = match literal {
            crate::semantic::Literal::Number(n) => *n,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::analyze;
    use crate::semantic::{AnalyzedExprKind, Constituent};

    #[test]
    fn test_extract_comparison_value_lemma() {
        let mut scope = Scope::new();
        scope.define("ονομα", GlossaType::String);

        // Analyze 'onomatos'
        let analysis = analyze("ὀνόματος");
        println!("Analysis: {:?}", analysis);

        let mut stmt = AssembledStatement::default();
        stmt.genitives.push(Constituent {
            lemma: smol_str::SmolStr::new(analysis.lemma.as_ref()),
            original: "ὀνόματος".into(),
            normalized: "ονοματος".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        let expr = extract_comparison_value(&stmt, &scope);
        if let AnalyzedExprKind::Variable(name) = expr.expr {
            assert_eq!(name, "ονομα", "Expected lemma 'ονομα', got '{}'", name);
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_extract_comparison_value_stripped() {
        let mut scope = Scope::new();
        scope.define("θ", GlossaType::Number);

        // 'thou' (θου) -> 'th' (θ)
        let mut stmt = AssembledStatement::default();
        stmt.genitives.push(Constituent {
            lemma: "dummy".into(), // Lemma lookup fails (not 'θ')
            original: "θου".into(),
            normalized: "θου".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        let expr = extract_comparison_value(&stmt, &scope);
        if let AnalyzedExprKind::Variable(name) = expr.expr {
            assert_eq!(name, "θ", "Expected stripped name 'θ', got '{}'", name);
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_extract_comparison_value_original() {
        let mut scope = Scope::new();
        // Define 'myos' (μυός) as a variable directly (maybe it's a genitive variable?)
        scope.define("μυος", GlossaType::Number);

        let mut stmt = AssembledStatement::default();
        stmt.genitives.push(Constituent {
            lemma: "mys".into(), // Lemma lookup fails (not 'μυος')
            original: "μυός".into(),
            normalized: "μυος".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        let expr = extract_comparison_value(&stmt, &scope);
        if let AnalyzedExprKind::Variable(name) = expr.expr {
            assert_eq!(
                name, "μυος",
                "Expected original name 'μυος', got '{}'",
                name
            );
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_extract_comparison_value_fallback() {
        let scope = Scope::new();
        // Nothing defined in scope. Should default to stripped name.

        let mut stmt = AssembledStatement::default();
        stmt.genitives.push(Constituent {
            lemma: "dummy".into(),
            original: "θου".into(),
            normalized: "θου".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        let expr = extract_comparison_value(&stmt, &scope);
        if let AnalyzedExprKind::Variable(name) = expr.expr {
            assert_eq!(
                name, "θ",
                "Expected fallback to stripped name 'θ', got '{}'",
                name
            );
        } else {
            panic!("Expected variable");
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::semantic::{AnalyzedExprKind, Constituent};
    #[test]
    fn test_detect_iterator_pattern_any() {
        // Covers process_explicit_quantifiers with scope lookup
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);

        let mut stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "list".into(),
                original: "list".into(),
                normalized: "list".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            ..Default::default()
        };
        // "any" quantifier logic is checked via flags, usually checked by lemma of subject
        // But here we manually trigger process_explicit_quantifiers by setting operators and genitives

        // Mock the quantifier logic by using "τι" (any) as an extra nominative
        // Subject must remain "list" for extract_collection to work (it rejects quantifiers)
        stmt.nominatives.push(Constituent {
            lemma: "τις".into(),
            original: "τι".into(),
            normalized: "τι".into(),
            case: crate::morphology::Case::Nominative,
            number: None,
            gender: None,
            person: None,
        });

        // "greater" (operator)
        stmt.operators
            .push(crate::morphology::lexicon::BinaryOp::Gt);

        // "than x" (genitive comparison value)
        stmt.genitives.push(Constituent {
            lemma: "x".into(),
            original: "x".into(), // Normalized will match scope "x"
            normalized: "x".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        // "print" verb to trigger detection
        stmt.verb = Some(crate::semantic::VerbConstituent {
            lemma: "λεγω".into(),
            original: "λεγε".into(),
            normalized: "λεγε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        });

        let result = detect_iterator_pattern(&stmt, &mut scope);
        assert!(result.is_ok());
        let expr_opt = result.unwrap();
        assert!(expr_opt.is_some());

        let expr = expr_opt.unwrap();
        // Should be MethodCall "any"
        if let AnalyzedExprKind::MethodCall { method, args, .. } = expr.expr {
            assert_eq!(method, "any");
            assert_eq!(args.len(), 1);
            // Verify argument is a closure x > x
            // (The detailed closure structure is complex to verify, but method name is key)
        } else {
            panic!("Expected MethodCall 'any', got {:?}", expr.expr);
        }
    }

    #[test]
    fn test_detect_iterator_pattern_find() {
        // Covers process_find with scope lookup
        let mut scope = Scope::new();
        scope.define("target", GlossaType::Number);

        let mut stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "list".into(),
                original: "list".into(),
                normalized: "list".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            ..Default::default()
        };

        // "greater" (operator)
        stmt.operators
            .push(crate::morphology::lexicon::BinaryOp::Gt);

        // "target" (genitive)
        stmt.genitives.push(Constituent {
            lemma: "target".into(),
            original: "target".into(),
            normalized: "target".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        // "find" verb
        stmt.verb = Some(crate::semantic::VerbConstituent {
            lemma: "ευρισκω".into(), // is_find_verb check
            original: "ευρε".into(),
            normalized: "ευρε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        });

        let result = detect_iterator_pattern(&stmt, &mut scope);
        assert!(result.is_ok());
        let expr_opt = result.unwrap();
        assert!(expr_opt.is_some());

        let expr = expr_opt.unwrap();
        // Should be MethodCall "find"
        if let AnalyzedExprKind::MethodCall { method, args, .. } = expr.expr {
            assert_eq!(method, "find");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected MethodCall 'find', got {:?}", expr.expr);
        }
    }

    #[test]
    fn test_detect_iterator_pattern_filter() {
        // Covers process_adjectives with scope lookup
        let mut scope = Scope::new();
        scope.define("threshold", GlossaType::Number);

        let mut stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "list".into(),
                original: "list".into(),
                normalized: "list".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            ..Default::default()
        };

        // "greater" (adjective -> filter)
        stmt.adjectives.push(Constituent {
            lemma: "μεγας".into(), // lemma for μείζον
            original: "μείζονα".into(),
            normalized: "μειζονα".into(),
            case: crate::morphology::Case::Accusative,
            number: None,
            gender: None,
            person: None,
        });

        // "threshold" (genitive comparison value)
        stmt.genitives.push(Constituent {
            lemma: "threshold".into(),
            original: "threshold".into(),
            normalized: "threshold".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        // "print" verb
        stmt.verb = Some(crate::semantic::VerbConstituent {
            lemma: "λεγω".into(),
            original: "λεγε".into(),
            normalized: "λεγε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        });

        let result = detect_iterator_pattern(&stmt, &mut scope);
        assert!(result.is_ok());
        let expr_opt = result.unwrap();
        assert!(expr_opt.is_some());

        let expr = expr_opt.unwrap();
        // Should be MethodCall "collect" (finalized), inner is filter
        if let AnalyzedExprKind::MethodCall {
            method, receiver, ..
        } = expr.expr
        {
            assert_eq!(method, "collect");
            // Check inner receiver
            if let AnalyzedExprKind::MethodCall {
                method: inner_method,
                ..
            } = receiver.expr
            {
                assert_eq!(inner_method, "filter");
            } else {
                panic!("Expected inner MethodCall 'filter'");
            }
        } else {
            panic!("Expected MethodCall 'collect'");
        }
    }

    #[test]
    fn test_try_parse_struct_instantiation_empty_terms() {
        let mut scope = Scope::new();
        let stmt = crate::ast::Statement::Regular {
            clauses: vec![crate::ast::Clause {
                expressions: vec![crate::ast::Expr::Phrase(vec![
                    crate::ast::Expr::Word(crate::ast::Word::new("var")),
                    crate::ast::Expr::Word(crate::ast::Word::new("νεον")),
                    crate::ast::Expr::Word(crate::ast::Word::new("Type")),
                    crate::ast::Expr::NumberLiteral(5),
                ])],
            }],
            is_query: false,
            is_propagate: false,
        };

        let result = try_parse_struct_instantiation(&stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
