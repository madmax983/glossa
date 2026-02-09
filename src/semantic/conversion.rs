//! Conversion from assembled statements to analyzed statements
//!
//! This module acts as the "interpreter" of the assembled semantic structure.
//! While the [`Assembler`](crate::semantic::Assembler) ensures grammatical correctness (Subject-Verb agreement),
//! this module assigns *meaning* to the grammatical structures.
//!
//! # The Interpreter Pattern
//!
//! The conversion process is essentially an interpretation step. It takes a
//! grammatically valid but semantically ambiguous "Assembled Statement" and
//! converts it into a typed, unambiguous "Analyzed Statement" (part of the HIR).
//!
//! This is where "word order independence" meets "semantic meaning".
//!
//! # Pattern Detection Strategy
//!
//! The [`classify_assembled_statement`] function uses a combination of strategies to
//! understand the statement's intent, checking patterns in a specific heuristic order:
//!
//! 1. **Pattern Delegation**: Complex patterns are delegated first.
//!    - **Iterator Chains**: `detect_iterator_pattern` (e.g., `list doubling print`).
//!    - **Property Access**: `classify_property_access_print` (e.g., `user.name print`).
//!    - **Struct Instantiation**: `try_parse_struct_instantiation` (e.g., `x new User ... let`).
//!    - **Function Calls**: `classify_function_call` (e.g., `my_func arg1 arg2 call`).
//!
//! 2. **Verb-Based Classification**: If no complex pattern matches, the main verb drives the logic.
//!    - **Binding** (`ἔστω`): `let x = value`.
//!    - **Assignment** (`γίγνεται`): `x = value`.
//!    - **Collection Ops** (`ὠθεῖ`, `ἕλκεται`, `τίθησι`): `push`, `pop`, `insert`.
//!    - **Print** (`λέγε`, `γράφε`): `println!`.
//!    - **Query** (`?`): Expressions ending in `?`.
//!
//! 3. **Expression Fallback**: If no verb implies a statement, it's treated as a pure expression.
//!    - **Operations**: `1 + 2`.
//!    - **Try/Propagate**: `expr;` (becomes `expr?`).

use super::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr, literal_to_type,
};
use super::patterns::detect_iterator_pattern;
use super::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, AssembledStatement, GlossaType, Scope,
};
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::morphology::{self};
use crate::semantic::assembled::{Constituent, Literal};
use crate::text::normalize_greek;

/// Convert an AssembledStatement to an AnalyzedStatement
///
/// This is the main entry point for lowering the "Assembled" semantic model (slot-based)
/// to the "Analyzed" model (HIR/AST-like).
///
/// # Arguments
///
/// * `asm_stmt` - The assembled statement from the `Assembler`.
/// * `scope` - The current semantic scope (for variable lookup and definition).
pub fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    classify_assembled_statement(asm_stmt, scope)
}

/// Classify an assembled statement and extract analyzed expressions
///
/// This function implements the heuristic priority list described in the module-level documentation.
pub fn classify_assembled_statement(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    if let Some(res) = classify_iterator_pattern(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_property_access_print(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_function_call(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_genitive_method_call(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_assertion(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_equality_assertion(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_subjunctive_comparison(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_variable_binding(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_assignment(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_collection_mutation(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_print(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = classify_query(asm_stmt, scope)? {
        return Ok(res);
    }

    classify_expression(asm_stmt)
}

// -------------------------------------------------------------------------------------------------
// Helper functions for classify_assembled_statement
// -------------------------------------------------------------------------------------------------

/// Helper: Detect iterator pattern
fn classify_iterator_pattern(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let has_find_or_print_verb = if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        crate::morphology::lexicon::is_print_verb(&verb_lemma)
            || crate::morphology::lexicon::is_find_verb(&verb_lemma)
    } else {
        false
    };

    if (!asm_stmt.participles.is_empty()
        || !asm_stmt.adjectives.is_empty()
        || has_find_or_print_verb)
        && let Some(analyzed) = detect_iterator_pattern(asm_stmt, scope)?
    {
        return Ok(Some(AnalyzedStatement::Print(vec![analyzed])));
    }

    Ok(None)
}

/// Helper: Detect property access print pattern (pi.xi)
fn classify_property_access_print(
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

/// Helper: Detect user-defined function call
fn classify_function_call(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        // Check if verb is a binding verb
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Check if object/nominative/genitive is a user-defined function
            let mut func_name = None;
            for nominative in &asm_stmt.nominatives {
                if scope.is_function(&nominative.lemma) {
                    func_name = Some(nominative.lemma.clone());
                    break;
                }
            }

            if func_name.is_none()
                && let Some(ref object) = asm_stmt.object
                && scope.is_function(&object.lemma)
            {
                func_name = Some(object.lemma.clone());
            }

            if func_name.is_none() {
                for genitive in &asm_stmt.genitives {
                    if scope.is_function(&genitive.lemma) {
                        func_name = Some(genitive.lemma.clone());
                        break;
                    }
                }
            }

            // If we found a function name, build the call
            if let Some(func) = func_name
                && let Some(ref subject) = asm_stmt.subject
            {
                let mut args: Vec<AnalyzedExpr> = asm_stmt
                    .literals
                    .iter()
                    .map(literal_to_analyzed_expr)
                    .collect();

                for nested_terms in &asm_stmt.nested_phrases {
                    let phrase_expr = Expr::Phrase(nested_terms.clone());
                    let analyzed = analyze_argument_expr(&phrase_expr, scope)?;
                    args.push(analyzed);
                }

                let return_type = scope
                    .lookup_function(&func)
                    .and_then(|sig| sig.return_type.clone())
                    .unwrap_or(GlossaType::Unknown);

                let func_call = AnalyzedExpr {
                    expr: AnalyzedExprKind::FunctionCall {
                        func: func.clone(),
                        args,
                    },
                    glossa_type: return_type.clone(),
                };

                let var_name = normalize_greek(&subject.original);
                scope.define(var_name.clone(), return_type.clone());

                return Ok(Some(AnalyzedStatement::Binding {
                    name: var_name.clone(),
                    value: func_call,
                    mutable: false,
                }));
            }
        }
    }
    Ok(None)
}

/// Helper: Detect subjunctive comparison (which looks like binding verb but isn't)
fn classify_subjunctive_comparison(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_binding_verb(&verb_lemma)
            && !asm_stmt.operators.is_empty()
            && !asm_stmt.literals.is_empty()
            && verb.mood == Some(crate::morphology::Mood::Subjunctive)
            && let Some(ref subject) = asm_stmt.subject
        {
            let left = if let Some(var_type) = scope.lookup(&subject.lemma) {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                    glossa_type: var_type.clone(),
                }
            } else {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(false),
                    glossa_type: GlossaType::Boolean,
                }
            };

            let right = literal_to_analyzed_expr(&asm_stmt.literals[0]);
            let op = asm_stmt.operators[0];
            let comparison = build_binary_expr(left, op, right);

            return Ok(Some(AnalyzedStatement::Expression(vec![comparison])));
        }
    }
    Ok(None)
}

/// Helper: Detect variable binding (let x = ...)
fn classify_variable_binding(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            let has_false_participle = !asm_stmt.participles.is_empty()
                && morphology::lexicon::lookup(&asm_stmt.participles[0].verb_lemma).is_none();

            let (var_name, actual_asm) = if has_false_participle {
                let first_participle = &asm_stmt.participles[0];
                let mut fixed_asm = asm_stmt.clone();
                fixed_asm.participles = asm_stmt.participles[1..].to_vec();
                (normalize_greek(&first_participle.original), fixed_asm)
            } else if let (Some(subject), Some(object)) = (&asm_stmt.subject, &asm_stmt.object) {
                let subject_name = normalize_greek(&subject.original);
                let object_name = normalize_greek(&object.original);

                if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
                    let mut swapped = asm_stmt.clone();
                    swapped.subject = Some(object.clone());
                    swapped.object = Some(subject.clone());
                    (object_name, swapped)
                } else {
                    (subject_name, asm_stmt.clone())
                }
            } else if let Some(subject) = &asm_stmt.subject {
                (normalize_greek(&subject.original), asm_stmt.clone())
            } else if !asm_stmt.participles.is_empty() {
                let first_participle = &asm_stmt.participles[0];
                let mut fixed_asm = asm_stmt.clone();
                fixed_asm.participles = asm_stmt.participles[1..].to_vec();
                (normalize_greek(&first_participle.original), fixed_asm)
            } else {
                return Err(GlossaError::semantic("Binding without subject"));
            };

            let (value_expr, value_type) = extract_value(&actual_asm, scope)?;

            let final_value_expr = if asm_stmt.is_propagate {
                AnalyzedExpr {
                    glossa_type: value_type.clone(),
                    expr: AnalyzedExprKind::Try(Box::new(value_expr)),
                }
            } else {
                value_expr
            };

            let is_mutable = asm_stmt.has_mutable_marker;
            if is_mutable {
                scope.define_mut(var_name.clone(), value_type.clone());
            } else {
                scope.define(var_name.clone(), value_type.clone());
            }

            return Ok(Some(AnalyzedStatement::Binding {
                name: var_name.clone(),
                value: final_value_expr,
                mutable: is_mutable,
            }));
        }
    }
    Ok(None)
}

/// Helper: Detect variable assignment (x = ...)
fn classify_assignment(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_assignment_verb(&verb_lemma) {
            let var_name = if let Some(ref subject) = asm_stmt.subject {
                normalize_greek(&subject.original)
            } else {
                return Err(GlossaError::semantic("Assignment without subject"));
            };

            let binding = scope.lookup_binding(&var_name);
            match binding {
                None => {
                    return Err(GlossaError::semantic(format!(
                        "Τὸ «{}» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό",
                        var_name
                    )));
                }
                Some(b) if !b.mutable => {
                    let _b = b; // Silence unused variable warning while keeping guard
                    return Err(GlossaError::semantic(crate::errors::immutable_assignment(
                        &var_name,
                    )));
                }
                Some(_b) => {
                    let has_value = !asm_stmt.literals.is_empty()
                        || asm_stmt.object.is_some()
                        || !asm_stmt.arrays.is_empty()
                        || !asm_stmt.unwraps.is_empty()
                        || !asm_stmt.index_accesses.is_empty()
                        || !asm_stmt.property_accesses.is_empty()
                        || !asm_stmt.nested_phrases.is_empty();

                    if !has_value {
                        return Err(GlossaError::semantic(format!(
                            "Τῇ πράξει «{} γίγνεται» δεῖ τιμῆς",
                            var_name
                        )));
                    }

                    let (value_expr, _) = extract_value(asm_stmt, scope)?;
                    scope.mark_used(&var_name);
                    return Ok(Some(AnalyzedStatement::Assignment {
                        name: var_name.clone(),
                        value: value_expr,
                    }));
                }
            }
        }
    }
    Ok(None)
}

/// Helper: Detect collection mutation (pop, push, insert)
fn classify_collection_mutation(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        // Pop
        if crate::morphology::lexicon::is_pop_verb(&verb_lemma)
            && let Some(ref subject) = asm_stmt.subject
        {
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                glossa_type: scope
                    .lookup(&subject.lemma)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown),
            };

            let method_call = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: "pop".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            };

            return Ok(Some(AnalyzedStatement::Expression(vec![method_call])));
        }

        // Push
        if crate::morphology::lexicon::is_push_verb(&verb_lemma)
            && let Some(ref subject) = asm_stmt.subject
        {
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                glossa_type: scope
                    .lookup(&subject.lemma)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown),
            };

            let arg = if let Some(lit) = asm_stmt.literals.first() {
                literal_to_analyzed_expr(lit)
            } else if let Some(ref obj) = asm_stmt.object {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                    glossa_type: scope
                        .lookup(&obj.lemma)
                        .cloned()
                        .unwrap_or(GlossaType::Unknown),
                }
            } else {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(0),
                    glossa_type: GlossaType::Number,
                }
            };

            let method_call = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: "push".into(),
                    args: vec![arg],
                },
                glossa_type: GlossaType::Unit,
            };

            return Ok(Some(AnalyzedStatement::Expression(vec![method_call])));
        }

        // Insert
        if crate::morphology::lexicon::is_insert_verb(&verb_lemma)
            && let Some(ref subject) = asm_stmt.subject
        {
            let subj_name = normalize_greek(&subject.original);
            let subj_type = scope
                .lookup(&subj_name)
                .cloned()
                .unwrap_or(GlossaType::Unknown);

            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj_name.clone()),
                glossa_type: subj_type.clone(),
            };

            let is_map = matches!(subj_type, GlossaType::Map(_, _));

            let args = if is_map && asm_stmt.literals.len() >= 2 {
                vec![
                    literal_to_analyzed_expr(&asm_stmt.literals[0]),
                    literal_to_analyzed_expr(&asm_stmt.literals[1]),
                ]
            } else if let Some(lit) = asm_stmt.literals.first() {
                vec![literal_to_analyzed_expr(lit)]
            } else if let Some(ref obj) = asm_stmt.object {
                vec![AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                    glossa_type: scope
                        .lookup(&obj.lemma)
                        .cloned()
                        .unwrap_or(GlossaType::Unknown),
                }]
            } else {
                vec![]
            };

            let return_type = if is_map {
                GlossaType::Option(Box::new(GlossaType::Unknown))
            } else {
                GlossaType::Boolean
            };
            let method_call = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: "insert".into(),
                    args,
                },
                glossa_type: return_type,
            };

            return Ok(Some(AnalyzedStatement::Expression(vec![method_call])));
        }
    }
    Ok(None)
}

/// Helper: Detect δεῖ assertion pattern
/// Pattern: <condition> δεῖ (any word order)
/// Examples: "2 ἐν χ δεῖ", "δεῖ 2 ἐν χ"
fn classify_assertion(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_assert_verb(&verb_lemma) {
            // The condition is everything except the verb
            // Common pattern: <element> ἐν <collection> δεῖ

            // Check for collection contains pattern (most common in tests)
            if asm_stmt.has_containment_preposition
                && let Some(ref subj) = asm_stmt.subject
            {
                // Pattern: element ἐν collection δεῖ
                let subj_name = normalize_greek(&subj.original);
                let collection_type = scope
                    .lookup(&subj_name)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown);

                let element = if let Some(lit) = asm_stmt.literals.first() {
                    literal_to_analyzed_expr(lit)
                } else {
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(0),
                        glossa_type: GlossaType::Number,
                    }
                };

                let is_map = matches!(collection_type, GlossaType::Map(_, _));
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
                        receiver: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(subj_name.clone()),
                            glossa_type: collection_type.clone(),
                        }),
                        method: method.into(),
                        args: vec![arg_expr],
                    },
                    glossa_type: GlossaType::Boolean,
                };

                let assert_expr = AnalyzedExpr {
                    expr: AnalyzedExprKind::Assert {
                        condition: Box::new(contains_expr),
                    },
                    glossa_type: GlossaType::Unit,
                };

                return Ok(Some(AnalyzedStatement::Expression(vec![assert_expr])));
            }
        }
    }
    Ok(None)
}

/// Helper: Detect ἰσοῦται equality assertion pattern
/// Pattern: <value1> <value2> ἰσοῦται (any word order)
/// Examples: "κ 5 ἰσοῦται", "ἰσοῦται κ 5", "5 κ ἰσοῦται"
fn classify_equality_assertion(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_equals_verb(&verb_lemma) {
            // We need two values to compare
            let mut left_expr = None;
            let mut right_expr = None;

            // Get subject (variable)
            if let Some(ref subj) = asm_stmt.subject
                && let Some(var_type) = scope.lookup(&subj.lemma)
            {
                left_expr = Some(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                });
            }

            // Get literal (expected value)
            if let Some(literal) = asm_stmt.literals.first() {
                right_expr = Some(literal_to_analyzed_expr(literal));
            }

            if let (Some(left), Some(right)) = (left_expr, right_expr) {
                let assert_eq_expr = AnalyzedExpr {
                    expr: AnalyzedExprKind::AssertEq {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    glossa_type: GlossaType::Unit,
                };

                return Ok(Some(AnalyzedStatement::Expression(vec![assert_eq_expr])));
            }
        }
    }
    Ok(None)
}

/// Helper: Detect print statement
fn classify_print(
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
fn classify_query(
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

/// Helper: Default expression
fn classify_expression(asm_stmt: &AssembledStatement) -> Result<AnalyzedStatement, GlossaError> {
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
fn try_parse_genitive_method_call(
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

/// Helper: Detect genitive method call (owner.method)
fn classify_genitive_method_call(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        if crate::morphology::lexicon::is_print_verb(&verb_lemma) {
            return Ok(None);
        }
    }

    if let Some((expr, _)) = try_parse_genitive_method_call(asm_stmt, scope) {
        return Ok(Some(AnalyzedStatement::Expression(vec![expr])));
    }

    Ok(None)
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
///
/// This function looks at the fields of the [`AssembledStatement`] and tries
/// to extract a single meaningful value from it. It prioritizes different kinds
/// of expressions in the following order:
///
/// 1. **Unwraps**: `expr!`
/// 2. **Enum Variants**: `Some(val)`, `Ok(val)`, `None` (on subject or nominatives)
/// 3. **Property Access**: `user.name`
/// 4. **Index Access**: `arr[0]`
/// 5. **Array Literals**: `[1, 2]`
/// 6. **Binary Operations**: `1 + 2`
/// 7. **Literals**: `42`, `"hello"`
/// 8. **Variables (Object)**: `x`
///
/// # Returns
///
/// Returns a tuple of `(AnalyzedExpr, GlossaType)`.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
    use crate::semantic::assembled::{Constituent, VerbConstituent};

    #[test]
    fn test_print_undefined_variable() {
        let mut scope = Scope::new();
        // Do NOT define "z" in scope

        let subj = Constituent {
            lemma: "z".into(),
            original: "z".into(),
            case: Case::Nominative,
            number: Some(Number::Singular),
            gender: Some(Gender::Neuter),
            person: None,
        };

        let verb = VerbConstituent {
            lemma: "λεγω".into(),
            original: "λέγε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        };

        let asm_stmt = AssembledStatement {
            subject: Some(subj),
            verb: Some(verb),
            ..Default::default()
        };

        let result = classify_assembled_statement(&asm_stmt, &mut scope).unwrap();

        match result {
            AnalyzedStatement::Print(exprs) => {
                assert_eq!(exprs.len(), 1);
                let expr = &exprs[0];
                match &expr.expr {
                    AnalyzedExprKind::Variable(name) => {
                        assert_eq!(name, "z");
                        assert_eq!(expr.glossa_type, GlossaType::Unknown);
                    }
                    _ => panic!("Expected Variable expression"),
                }
            }
            _ => panic!("Expected Print statement"),
        }
    }
}
