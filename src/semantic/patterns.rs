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

use super::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
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
            let normalized_adj = normalize_greek(&adj_word.normalized);
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

                    let analyzed_arg =
                        crate::semantic::expressions::analyze_argument_expr(term, scope)?;

                    // Handle special case: String literal for String type needs .to_string() wrap
                    let is_string_literal = matches!(term, Expr::StringLiteral(_));
                    let final_arg = if is_string_literal
                        && matches!(expected_type, GlossaType::String)
                    {
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::MethodCall {
                                receiver: Box::new(analyzed_arg),
                                method: "to_string".into(),
                                args: vec![],
                            },
                            glossa_type: GlossaType::String,
                        }
                    } else {
                        analyzed_arg
                    };

                    args.push(final_arg);
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
