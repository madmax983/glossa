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
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::morphology::{self};
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use crate::semantic::resolver::Scope;

use crate::semantic::types::GlossaType;
use crate::semantic::{Constituent, Literal};

/// Convert an AssembledStatement to an AnalyzedStatement
///
/// This is the main entry point for lowering the "Assembled" semantic model (slot-based)
/// to the "Analyzed" model (HIR/AST-like).
///
/// Evaluates and translates a grammatically sound statement into the semantically typed AST.
///
/// This serves as the top-level interpreter connecting the raw output of the
/// [`crate::semantic::Assembler`] (`AssembledStatement`) with the High-Level Intermediate
/// Representation (`AnalyzedStatement`). It assigns concrete meaning to grammatical roles
/// (e.g., assigning a Subject the role of "Variable Name").
///
/// # Arguments
///
/// * `asm_stmt` - The assembled statement from the `Assembler`.
/// * `scope` - The current semantic scope (for variable lookup and definition).
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::semantic::assembly::AssembledStatement;
/// use glossa::semantic::conversion::convert_assembled_to_analyzed;
/// use glossa::semantic::resolver::Scope;
/// use glossa::ast::{Expr, Word};
/// use glossa::morphology::lexicon::{LexiconEntry, VerbType};
///
/// let mut scope = Scope::new();
/// let mut asm = AssembledStatement::new();
///
/// // Simulate: "«χαῖρε» λέγε."
/// asm.verb = Some(LexiconEntry {
///     lemma: "λεγω".into(),
///     english_equivalent: "say".into(),
///     part_of_speech: glossa::morphology::lexicon::PartOfSpeech::Verb(VerbType::Transitive),
/// });
/// asm.strings.push("χαῖρε".into());
///
/// let result = convert_assembled_to_analyzed(&asm, &mut scope);
/// assert!(result.is_ok());
/// ```
pub fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    classify_assembled_statement(asm_stmt, scope)
}

/// Diagnoses the semantic intent of an assembled statement using heuristic pattern matching.
///
/// The assembler organizes sentences by grammatical slots (subject, verb, object), but does
/// not understand *what* the sentence is trying to do (e.g., is it an assignment, a print call,
/// or a function invocation?). This classification must happen first, as a value extraction
/// for a Binding operates entirely differently from a Function Call argument extraction.
///
/// This implements the prioritized heuristic order defined in the module-level documentation.
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::semantic::assembly::AssembledStatement;
/// use glossa::semantic::conversion::classify_assembled_statement;
/// use glossa::semantic::resolver::Scope;
/// use glossa::ast::{Expr, Word};
/// use glossa::morphology::lexicon::{LexiconEntry, VerbType};
///
/// let mut scope = Scope::new();
/// let mut asm = AssembledStatement::new();
///
/// // Represents: "«χαῖρε» λέγε." (Say "hello")
/// asm.verb = Some(LexiconEntry {
///     lemma: "λεγω".into(),
///     english_equivalent: "say".into(),
///     part_of_speech: glossa::morphology::lexicon::PartOfSpeech::Verb(VerbType::Transitive),
/// });
/// asm.strings.push("χαῖρε".into());
///
/// let stmt = classify_assembled_statement(&asm, &mut scope).unwrap();
///
/// match stmt {
///     glossa::semantic::AnalyzedStatement::Print { .. } => assert!(true),
///     _ => panic!("Expected Print statement"),
/// }
/// ```
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

    classify_expression(asm_stmt, scope)
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
        let verb_lemma = &verb.lemma;
        crate::morphology::lexicon::is_print_verb(verb_lemma)
            || crate::morphology::lexicon::is_find_verb(verb_lemma)
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
    let Some(verb) = &asm_stmt.verb else {
        return Ok(None);
    };

    if !crate::morphology::lexicon::is_print_verb(&verb.lemma) {
        return Ok(None);
    }

    if asm_stmt.genitives.is_empty() {
        return Ok(None);
    }

    let Some(subject) = &asm_stmt.subject else {
        return Ok(None);
    };

    // Get owner from genitive (use lemma to get base variable name)
    let owner_lemma = &asm_stmt.genitives[0].lemma;

    // Get property from subject (nominative)
    let property = &subject.normalized;

    // Check if owner is a struct type in scope
    let Some(owner_type) = scope.lookup(owner_lemma) else {
        return Ok(None);
    };

    if !matches!(owner_type, GlossaType::Struct { .. }) {
        return Ok(None);
    }

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

    Ok(Some(AnalyzedStatement::Print(vec![prop_access])))
}

/// Helper: Detect user-defined function call
fn classify_function_call(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(verb) = &asm_stmt.verb else {
        return Ok(None);
    };

    // Check if verb is a binding verb
    if !crate::morphology::lexicon::is_binding_verb(&verb.lemma) {
        return Ok(None);
    }

    let Some(subject) = &asm_stmt.subject else {
        return Ok(None);
    };

    // If we found a function name, build the call
    let Some(func) = resolve_function_name(asm_stmt, scope) else {
        return Ok(None);
    };

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

    let var_name = &subject.normalized;
    scope.define(var_name.clone(), return_type.clone());

    Ok(Some(AnalyzedStatement::Binding {
        name: var_name.clone(),
        value: func_call,
        mutable: false,
    }))
}

/// Helper: Resolve the function name from an assembled statement
fn resolve_function_name(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Option<smol_str::SmolStr> {
    for nominative in &asm_stmt.nominatives {
        if scope.is_function(&nominative.lemma) {
            return Some(nominative.lemma.clone());
        }
    }

    if let Some(ref object) = asm_stmt.object
        && scope.is_function(&object.lemma)
    {
        return Some(object.lemma.clone());
    }

    for genitive in &asm_stmt.genitives {
        if scope.is_function(&genitive.lemma) {
            return Some(genitive.lemma.clone());
        }
    }

    None
}

/// Helper: Detect subjunctive comparison (which looks like binding verb but isn't)
fn classify_subjunctive_comparison(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(verb) = &asm_stmt.verb else {
        return Ok(None);
    };

    if !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
        || asm_stmt.operators.is_empty()
        || asm_stmt.literals.is_empty()
        || verb.mood != Some(crate::morphology::Mood::Subjunctive)
    {
        return Ok(None);
    }

    let Some(subject) = &asm_stmt.subject else {
        return Ok(None);
    };

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

    Ok(Some(AnalyzedStatement::Expression(vec![comparison])))
}

/// Helper: Resolve the target variable name and the effective assembled statement for binding
///
/// ⚡ Bolt Optimization: Returns a `std::borrow::Cow<'_, AssembledStatement>` instead of
/// `AssembledStatement` to avoid cloning a large struct on a hot path during semantic analysis.
/// We only need to clone and mutate the assembled statement if we're swapping subject/object
/// or fixing false participles. Otherwise, we just return a borrowed reference to the original statement.
fn resolve_binding_target<'a>(
    asm_stmt: &'a AssembledStatement,
    scope: &Scope,
) -> Result<(String, std::borrow::Cow<'a, AssembledStatement>), GlossaError> {
    // Check for "false participles" (nouns misclassified as participles)
    let has_false_participle = !asm_stmt.participles.is_empty()
        && morphology::lexicon::lookup(&asm_stmt.participles[0].verb_lemma).is_none();

    if has_false_participle {
        let first_participle = &asm_stmt.participles[0];
        let mut fixed_asm = asm_stmt.clone();
        fixed_asm.participles = asm_stmt.participles[1..].to_vec();
        return Ok((
            first_participle.normalized.to_string(),
            std::borrow::Cow::Owned(fixed_asm),
        ));
    }

    // Check for Subject/Object swap (if Subject is defined and Object is not, bind to Object)
    if let (Some(subject), Some(object)) = (&asm_stmt.subject, &asm_stmt.object) {
        let subject_name = &subject.normalized;
        let object_name = &object.normalized;

        if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
            let mut swapped = asm_stmt.clone();
            swapped.subject = Some(object.clone());
            swapped.object = Some(subject.clone());
            return Ok((object_name.to_string(), std::borrow::Cow::Owned(swapped)));
        } else {
            return Ok((
                subject_name.to_string(),
                std::borrow::Cow::Borrowed(asm_stmt),
            ));
        }
    }

    // Default case: Bind to Subject
    if let Some(subject) = &asm_stmt.subject {
        return Ok((
            subject.normalized.to_string(),
            std::borrow::Cow::Borrowed(asm_stmt),
        ));
    }

    // Fallback: Bind to first participle (if any remain)
    if !asm_stmt.participles.is_empty() {
        let first_participle = &asm_stmt.participles[0];
        let mut fixed_asm = asm_stmt.clone();
        fixed_asm.participles = asm_stmt.participles[1..].to_vec();
        return Ok((
            first_participle.normalized.to_string(),
            std::borrow::Cow::Owned(fixed_asm),
        ));
    }

    Err(GlossaError::semantic("Binding without subject"))
}

/// Helper: Detect variable binding (let x = ...)
fn classify_variable_binding(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(verb) = &asm_stmt.verb else {
        return Ok(None);
    };

    if !crate::morphology::lexicon::is_binding_verb(&verb.lemma) {
        return Ok(None);
    }

    let (var_name, actual_asm) = resolve_binding_target(asm_stmt, scope)?;
    let (value_expr, value_type) = extract_value(actual_asm.as_ref(), scope)?;

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

    Ok(Some(AnalyzedStatement::Binding {
        name: var_name.into(),
        value: final_value_expr,
        mutable: is_mutable,
    }))
}

/// Helper: Detect variable assignment (x = ...)
fn classify_assignment(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(verb) = &asm_stmt.verb else {
        return Ok(None);
    };

    if !crate::morphology::lexicon::is_assignment_verb(&verb.lemma) {
        return Ok(None);
    }

    let Some(subject) = &asm_stmt.subject else {
        return Err(GlossaError::semantic("Assignment without subject"));
    };

    let var_name = subject.normalized.clone();

    match scope.lookup_binding(&var_name) {
        None => Err(GlossaError::semantic(format!(
            "Τὸ «{}» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό",
            var_name
        ))),
        Some(b) if !b.mutable => Err(GlossaError::semantic(format!(
            "Τὸ «{}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ",
            &var_name
        ))),
        Some(_) => {
            let has_value = !asm_stmt.literals.is_empty()
                || asm_stmt.object.is_some()
                || !asm_stmt.arrays.is_empty()
                || !asm_stmt.unwraps.is_empty()
                || !asm_stmt.index_accesses.is_empty()
                || !asm_stmt.property_accesses.is_empty()
                || !asm_stmt.nested_phrases.is_empty();

            if !has_value {
                return Err(GlossaError::semantic(format!(
                    "Τῇ πράξει «{} γίγνεται» δεῖ τιμῆς (Assignment requires a value)",
                    var_name
                )));
            }

            let (value_expr, _) = extract_value(asm_stmt, scope)?;
            scope.mark_used(&var_name);
            Ok(Some(AnalyzedStatement::Assignment {
                name: var_name.clone(),
                value: value_expr,
            }))
        }
    }
}

/// Helper: Detect collection mutation (pop, push, insert)
fn classify_collection_mutation(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(ref verb) = asm_stmt.verb else {
        return Ok(None);
    };

    let verb_lemma = &verb.lemma;

    if let Some(res) = classify_pop(verb_lemma, asm_stmt, scope)? {
        return Ok(Some(res));
    }
    if let Some(res) = classify_push(verb_lemma, asm_stmt, scope)? {
        return Ok(Some(res));
    }
    if let Some(res) = classify_insert(verb_lemma, asm_stmt, scope)? {
        return Ok(Some(res));
    }

    Ok(None)
}

fn classify_pop(
    verb_lemma: &str,
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if !crate::morphology::lexicon::is_pop_verb(verb_lemma) {
        return Ok(None);
    }

    let Some(ref subject) = asm_stmt.subject else {
        return Ok(None);
    };

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

    Ok(Some(AnalyzedStatement::Expression(vec![method_call])))
}

fn classify_push(
    verb_lemma: &str,
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if !crate::morphology::lexicon::is_push_verb(verb_lemma) {
        return Ok(None);
    }

    let Some(ref subject) = asm_stmt.subject else {
        return Ok(None);
    };

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

    Ok(Some(AnalyzedStatement::Expression(vec![method_call])))
}

fn classify_insert(
    verb_lemma: &str,
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if !crate::morphology::lexicon::is_insert_verb(verb_lemma) {
        return Ok(None);
    }

    let Some(ref subject) = asm_stmt.subject else {
        return Ok(None);
    };

    let subj_name = &subject.normalized;
    let subj_type = scope
        .lookup(subj_name)
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

    Ok(Some(AnalyzedStatement::Expression(vec![method_call])))
}

/// Helper: Detect δεῖ assertion pattern
/// Pattern: `<condition>` δεῖ (any word order)
/// Examples: "2 ἐν χ δεῖ", "δεῖ 2 ἐν χ"
fn classify_assertion(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(ref verb) = asm_stmt.verb else {
        return Ok(None);
    };

    let verb_lemma = &verb.lemma;
    if !crate::morphology::lexicon::is_assert_verb(verb_lemma) {
        return Ok(None);
    }

    // The condition is everything except the verb
    // Common pattern: <element> ἐν <collection> δεῖ

    // Check for collection contains pattern (most common in tests)
    if !asm_stmt.has_containment_preposition {
        return Ok(None);
    }

    let Some(ref subj) = asm_stmt.subject else {
        return Ok(None);
    };

    // Pattern: element ἐν collection δεῖ
    let subj_name = &subj.normalized;
    let collection_type = scope
        .lookup(subj_name)
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

    Ok(Some(AnalyzedStatement::Expression(vec![assert_expr])))
}

/// Helper: Detect ἰσοῦται equality assertion pattern
/// Pattern: `<value1>` `<value2>` ἰσοῦται (any word order)
/// Examples: "κ 5 ἰσοῦται", "ἰσοῦται κ 5", "5 κ ἰσοῦται"
fn classify_equality_assertion(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    let Some(ref verb) = asm_stmt.verb else {
        return Ok(None);
    };

    let verb_lemma = &verb.lemma;
    if !crate::morphology::lexicon::is_equals_verb(verb_lemma) {
        return Ok(None);
    }

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

    let (Some(left), Some(right)) = (left_expr, right_expr) else {
        return Ok(None);
    };

    let assert_eq_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::AssertEq {
            left: Box::new(left),
            right: Box::new(right),
        },
        glossa_type: GlossaType::Unit,
    };

    Ok(Some(AnalyzedStatement::Expression(vec![assert_eq_expr])))
}

fn try_print_binary_op(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Option<Vec<AnalyzedExpr>> {
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
            return Some(vec![bin_expr]);
        }
    }
    None
}

fn try_print_property_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Option<Vec<AnalyzedExpr>> {
    if !asm_stmt.property_accesses.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.property_accesses.len());
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
        return Some(args);
    }
    None
}

fn try_print_index_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<Vec<AnalyzedExpr>>, GlossaError> {
    if !asm_stmt.index_accesses.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.index_accesses.len());
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
        return Ok(Some(args));
    }
    Ok(None)
}

fn try_print_unwrap(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<Vec<AnalyzedExpr>>, GlossaError> {
    if !asm_stmt.unwraps.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.unwraps.len());
        for unwrap_expr in &asm_stmt.unwraps {
            let inner_analyzed = analyze_argument_expr(unwrap_expr, scope)?;
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(inner_analyzed)),
                glossa_type: GlossaType::Unknown,
            });
        }
        return Ok(Some(args));
    }
    Ok(None)
}

fn try_print_default(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Vec<AnalyzedExpr>, GlossaError> {
    let mut args =
        build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators)?;

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

    Ok(args)
}

/// Helper: Detect print statement
fn classify_print(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = &verb.lemma;

        if crate::morphology::lexicon::is_print_verb(verb_lemma) {
            if let Some(args) = try_print_binary_op(asm_stmt, scope) {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            if let Some(args) = try_print_property_access(asm_stmt, scope) {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            if let Some(args) = try_print_index_access(asm_stmt, scope)? {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            if let Some(args) = try_print_unwrap(asm_stmt, scope)? {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            let args = try_print_default(asm_stmt, scope)?;
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
    if !asm_stmt.is_query {
        return Ok(None);
    }

    if let Some(analyzed) = classify_containment_query(asm_stmt, scope)? {
        return Ok(Some(analyzed));
    }

    // Regular query
    let mut exprs = Vec::with_capacity(asm_stmt.literals.len() + 1);
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
    Ok(Some(AnalyzedStatement::Query(exprs)))
}

/// Helper: Detect containment queries
fn classify_containment_query(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if !asm_stmt.has_containment_preposition {
        return Ok(None);
    }

    let Some(ref subj) = asm_stmt.subject else {
        return Ok(None);
    };

    let subj_name = &subj.normalized;
    let subj_type = scope
        .lookup(subj_name)
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

    Ok(Some(AnalyzedStatement::Query(vec![contains_expr])))
}

/// Helper: Default expression
#[allow(unused_variables)]
fn classify_expression(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    // Determine if we should attempt to build expressions from literals+operators
    // or if we are in a fallback scenario (using Subject/Object with operators).
    // If literals < operators + 1, build_expressions_from_literals_and_ops will fail.
    // In that case, we only build literals and let the fallback logic handle operators.

    let (literals_to_build, operators_to_build) = if !asm_stmt.operators.is_empty()
        && asm_stmt.literals.len() < asm_stmt.operators.len() + 1
    {
        // Fallback case: operators likely depend on Subject/Object
        (asm_stmt.literals.as_slice(), &[][..])
    } else {
        (asm_stmt.literals.as_slice(), asm_stmt.operators.as_slice())
    };

    let mut exprs = build_expressions_from_literals_and_ops(literals_to_build, operators_to_build)?;

    // If we have operators but couldn't build a full expression from literals alone (usually implies literals < 2),
    // we should look for Subject/Object to complete the binary expression.
    if !asm_stmt.operators.is_empty() && asm_stmt.literals.len() < 2 {
        let op = asm_stmt.operators[0];

        let left = asm_stmt.subject.as_ref().map(|subj| AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
            glossa_type: GlossaType::Unknown,
        });

        // Try to get right operand from exprs (literal) or object or nominatives
        let right = if let Some(lit_expr) = exprs.first() {
            Some(lit_expr.clone())
        } else if let Some(ref obj) = asm_stmt.object {
            Some(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            })
        } else {
            asm_stmt.nominatives.first().map(|nom| AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(nom.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            })
        };

        if let (Some(l), Some(r)) = (left, right) {
            let bin_expr = build_binary_expr(l, op, r);
            exprs = vec![bin_expr];
        }
    }

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
        #[allow(clippy::collapsible_if)]
        if let Some(last_expr) = exprs.pop() {
            let try_expr = AnalyzedExpr {
                glossa_type: last_expr.glossa_type.clone(),
                expr: AnalyzedExprKind::Try(Box::new(last_expr)),
            };
            exprs.push(try_expr);
        }
    }

    Ok(AnalyzedStatement::Expression(exprs))
}

/// Helper: Common logic for genitive method call parsing
fn try_parse_genitive_method_call(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Option<(AnalyzedExpr, GlossaType)> {
    let subject = asm_stmt.subject.as_ref()?;

    if asm_stmt.genitives.is_empty() {
        return None;
    }

    let owner_lemma = &asm_stmt.genitives[0].lemma;
    let method_name = &subject.normalized;

    let owner_type = scope.lookup(owner_lemma)?;

    if scope.is_defined(method_name) {
        return None;
    }

    let receiver = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(owner_lemma.clone()),
        glossa_type: owner_type.clone(),
    };

    let mut args = Vec::with_capacity(asm_stmt.literals.len());
    for lit in &asm_stmt.literals {
        args.push(literal_to_analyzed_expr(lit));
    }

    Some((
        AnalyzedExpr {
            expr: AnalyzedExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: method_name.clone(),
                args,
            },
            glossa_type: GlossaType::Unknown,
        },
        GlossaType::Unknown,
    ))
}

/// Helper: Detect genitive method call (owner.method)
fn classify_genitive_method_call(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = &verb.lemma;
        if crate::morphology::lexicon::is_print_verb(verb_lemma) {
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
    let lemma = &word.lemma;
    let original = &word.normalized;

    // Helper to check if a word matches a predicate
    let check = |pred: fn(&str) -> bool| pred(lemma) || pred(original);

    // None
    if check(crate::morphology::lexicon::is_none_word) {
        return Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::None,
                glossa_type: GlossaType::Option(Box::new(GlossaType::Unknown)),
            },
            GlossaType::Option(Box::new(GlossaType::Unknown)),
        ));
    }

    if let Some(lit) = literals.first() {
        let inner_expr = literal_to_analyzed_expr(lit);
        let inner_type = inner_expr.glossa_type.clone();

        // Some
        if check(crate::morphology::lexicon::is_some_word) {
            return Some((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Some(Box::new(inner_expr)),
                    glossa_type: GlossaType::Option(Box::new(inner_type.clone())),
                },
                GlossaType::Option(Box::new(inner_type)),
            ));
        }

        // Ok
        if check(crate::morphology::lexicon::is_ok_word) {
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
        if check(crate::morphology::lexicon::is_err_word) {
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
    }

    None
}

// -------------------------------------------------------------------------------------------------
// Helper functions for extract_value
// -------------------------------------------------------------------------------------------------

fn extract_unwrap(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if !asm_stmt.unwraps.is_empty() {
        let inner_analyzed = analyze_argument_expr(&asm_stmt.unwraps[0], scope)?;
        return Ok(Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(inner_analyzed)),
                glossa_type: GlossaType::Unknown, // Type will be inferred
            },
            GlossaType::Unknown,
        )));
    }
    Ok(None)
}

fn extract_enum_from_subject(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref subj) = asm_stmt.subject
        && let Some(result) = detect_enum_variant(subj, &asm_stmt.literals)
    {
        return Ok(Some(result));
    }
    Ok(None)
}

fn extract_genitive_method(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(result) = try_parse_genitive_method_call(asm_stmt, scope) {
        return Ok(Some(result));
    }
    Ok(None)
}

fn extract_enum_from_nominatives(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    for nom in &asm_stmt.nominatives {
        if let Some(result) = detect_enum_variant(nom, &asm_stmt.literals) {
            return Ok(Some(result));
        }
    }
    Ok(None)
}

fn extract_property_access(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some((owner, method)) = asm_stmt.property_accesses.first() {
        let receiver = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(owner.clone().into()),
            glossa_type: GlossaType::Unknown,
        };
        return Ok(Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: method.clone().into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Number,
            },
            GlossaType::Number,
        )));
    }
    Ok(None)
}

fn extract_index_access(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some((array_expr, index_expr)) = asm_stmt.index_accesses.first() {
        let array_analyzed = analyze_argument_expr(array_expr, scope)?;
        let index_analyzed = analyze_argument_expr(index_expr, scope)?;
        return Ok(Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(array_analyzed),
                    index: Box::new(index_analyzed),
                },
                glossa_type: GlossaType::Unknown, // Element type is unknown without inference
            },
            GlossaType::Unknown,
        )));
    }
    Ok(None)
}

fn extract_array(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(array_elements) = asm_stmt.arrays.first() {
        let mut analyzed_elements = Vec::with_capacity(array_elements.len());
        for e in array_elements {
            analyzed_elements.push(analyze_argument_expr(e, scope)?);
        }

        let element_type = analyzed_elements
            .first()
            .map(|e| e.glossa_type.clone())
            .unwrap_or(GlossaType::Unknown);
        return Ok(Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(analyzed_elements),
                glossa_type: GlossaType::List(Box::new(element_type)),
            },
            GlossaType::List(Box::new(GlossaType::Unknown)),
        )));
    }
    Ok(None)
}

fn extract_binary_op(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if asm_stmt.operators.is_empty() {
        return Ok(None);
    }

    // Check if we can build from literals alone (2+ literals)
    if asm_stmt.literals.len() >= 2 {
        let exprs =
            build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators)?;
        if let Some(expr) = exprs.into_iter().next() {
            let ty = expr.glossa_type.clone();
            return Ok(Some((expr, ty)));
        }
    }

    let make_var = |lemma: &smol_str::SmolStr| AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(lemma.clone()),
        glossa_type: scope.lookup(lemma).cloned().unwrap_or(GlossaType::Unknown),
    };

    let op = asm_stmt.operators[0];

    // Or check if we can combine object + literal with operator
    if let Some(ref obj) = asm_stmt.object {
        if !asm_stmt.literals.is_empty() {
            // Build: object op literal
            let left = make_var(&obj.lemma);
            let right = literal_to_analyzed_expr(&asm_stmt.literals[0]);
            let bin_expr = build_binary_expr(left, op, right);
            let ty = bin_expr.glossa_type.clone();
            return Ok(Some((bin_expr, ty)));
        }

        // Object + Nominative (e.g. x + y)
        if let Some(nom) = asm_stmt.nominatives.first() {
            let left = make_var(&obj.lemma);
            let right = make_var(&nom.lemma);
            let bin_expr = build_binary_expr(left, op, right);
            let ty = bin_expr.glossa_type.clone();
            return Ok(Some((bin_expr, ty)));
        }
    }

    // Nominative + Nominative (e.g. a + b, where both are extra nominatives)
    if asm_stmt.nominatives.len() >= 2 {
        let left = make_var(&asm_stmt.nominatives[0].lemma);
        let right = make_var(&asm_stmt.nominatives[1].lemma);
        let bin_expr = build_binary_expr(left, op, right);
        let ty = bin_expr.glossa_type.clone();
        return Ok(Some((bin_expr, ty)));
    }

    Ok(None)
}

fn extract_literal(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(lit) = asm_stmt.literals.first() {
        return Ok(Some((literal_to_analyzed_expr(lit), literal_to_type(lit))));
    }
    Ok(None)
}

fn extract_enum_from_object(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref obj) = asm_stmt.object {
        return Ok(detect_enum_variant(obj, &asm_stmt.literals));
    }
    Ok(None)
}

fn extract_subject_fallback(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref subj) = asm_stmt.subject {
        let subj_lemma = &subj.lemma;

        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(subj_lemma) {
            return Ok(Some((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            )));
        }

        if !scope.is_defined(subj_lemma) {
            return Err(GlossaError::undefined(subj_lemma.as_str()));
        }

        return Ok(Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            },
            GlossaType::Unknown,
        )));
    }
    Ok(None)
}

fn extract_object_fallback(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref obj) = asm_stmt.object {
        let obj_lemma = &obj.lemma;

        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(obj_lemma) {
            return Ok(Some((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            )));
        }

        if !scope.is_defined(obj_lemma) {
            return Err(GlossaError::undefined(obj_lemma.as_str()));
        }

        return Ok(Some((
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            },
            GlossaType::Unknown,
        )));
    }
    Ok(None)
}

/// Extract value from assembled statement
///
/// This function looks at the fields of the [`AssembledStatement`] and tries
/// to extract a single meaningful value from it. It prioritizes different kinds
/// of expressions in the following order:
///
/// 1. **Unwraps**: `expr!`
/// 2. **Enum Variants**: `Some(val)`, `Ok(val)`, `None` (on subject or nominatives)
/// 3. **Genitive Methods**: `owner.method`
/// 4. **Property Access**: `user.name`
/// 5. **Index Access**: `arr[0]`
/// 6. **Array Literals**: `[1, 2]`
/// 7. **Binary Operations**: `1 + 2`
/// 8. **Object Enum Variants**: `Some(val)`, `Ok(val)`, `None` (on object) - *Prioritized over literals*
/// 9. **Literals**: `42`, `"hello"`
/// 10. **Variables (Object)**: `x`
///
/// Consolidates scattering values (numbers, strings, blocks) into a single logical expression.
///
/// In GLOSSA, depending on the sentence phrasing, the "value" of an assignment might be located
/// in the subject slot, an explicit number literal slot, a string slot, or nested inside a phrase.
/// This function acts as a semantic vacuum, pulling out the first valid expression value it can find
/// in the statement regardless of where the `Assembler` categorized it grammatically.
///
/// # Returns
///
/// * `Ok((AnalyzedExpr, GlossaType))` containing the resolved expression and its inferred type.
/// * `Err(GlossaError)` if no valid value expression can be identified.
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::semantic::assembly::AssembledStatement;
/// use glossa::semantic::conversion::extract_value;
/// use glossa::semantic::resolver::Scope;
/// use glossa::semantic::types::GlossaType;
/// use glossa::semantic::AnalyzedExprKind;
///
/// let scope = Scope::new();
/// let mut asm = AssembledStatement::new();
///
/// // Simulate a statement that contains a number literal: 42
/// asm.numbers.push(42);
///
/// let (expr, ty) = extract_value(&asm, &scope).unwrap();
///
/// assert_eq!(ty, GlossaType::Number);
/// match expr.expr {
///     AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 42),
///     _ => panic!("Expected NumberLiteral"),
/// }
/// ```
pub fn extract_value(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<(AnalyzedExpr, GlossaType), GlossaError> {
    if !asm_stmt.nested_phrases.is_empty() {
        // Handle nested phrases (parenthesized expressions) which act as values
        // Usually there is only one for a value expression
        if let Some(terms) = asm_stmt.nested_phrases.first() {
            let phrase_expr = Expr::Phrase(terms.clone());
            // Analyze with recursion depth check reset (as it's a new analysis root)
            let analyzed = analyze_argument_expr(&phrase_expr, scope)?;
            let ty = analyzed.glossa_type.clone();
            return Ok((analyzed, ty));
        }
    }

    if !asm_stmt.blocks.is_empty() {
        // Handle blocks (braced expressions) which act as values
        if let Some(stmts) = asm_stmt.blocks.first() {
            let block_expr = Expr::Block(stmts.clone());
            // Analyze with recursion depth check reset (as it's a new analysis root)
            // Note: analyze_argument_expr will call analyze_block, which now enforces single-statement logic
            let analyzed = analyze_argument_expr(&block_expr, scope)?;
            let ty = analyzed.glossa_type.clone();
            return Ok((analyzed, ty));
        }
    }

    if let Some(res) = extract_unwrap(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_enum_from_subject(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_genitive_method(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_enum_from_nominatives(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_property_access(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_index_access(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_array(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_binary_op(asm_stmt, scope)? {
        return Ok(res);
    }
    // Fix: Check object for enum variants BEFORE literals to avoid shadowing Some(literal) by literal
    if let Some(res) = extract_enum_from_object(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_literal(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_object_fallback(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_subject_fallback(asm_stmt, scope)? {
        return Ok(res);
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

    #[test]
    fn test_classify_pop_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(), // not a pop verb
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_pop("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_pop_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ὠθεῖ".into(), // actually a push verb, let's use ἕλκεται for pop, but any string works for the missing subject test since it checks lemma first
                normalized: "ἕλκεται".into(),
                original: "ἕλκεται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        // The check inside classify_pop explicitly looks at the passed verb_lemma ("ἕλκεται" is pop)
        let result = classify_pop("ἕλκεται", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_push_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_push_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ὠθεῖ".into(),
                normalized: "ὠθεῖ".into(),
                original: "ὠθεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("ὠθεῖ", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_insert_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_insert_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τίθησι".into(),
                normalized: "τίθησι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τίθησι", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_no_containment() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "δεῖ".into(),
                normalized: "δεῖ".into(),
                original: "δεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            has_containment_preposition: false,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "δεῖ".into(),
                normalized: "δεῖ".into(),
                original: "δεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            has_containment_preposition: true,
            subject: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_missing_left_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None, // Missing subject means left_expr will be None
            literals: vec![Literal::Number(5)],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_undefined_left_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "y".into(), // y is not defined in scope
                normalized: "y".into(),
                original: "y".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![Literal::Number(5)],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_missing_right_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![], // Empty literals means right_expr will be None
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_collection_mutation_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_collection_mutation(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_print_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_print_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(), // not a print verb (λέγε is, but here testing the literal check)
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_print_binary_op_empty() {
        let asm_stmt = AssembledStatement {
            operators: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_binary_op(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_print_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_property_access(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_print_index_access_empty() {
        let asm_stmt = AssembledStatement {
            index_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_index_access(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_print_unwrap_empty() {
        let asm_stmt = AssembledStatement {
            unwraps: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_unwrap(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_no_subject() {
        let asm_stmt = AssembledStatement {
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_no_genitives() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_owner_not_found() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![Constituent {
                lemma: "x".into(), // Not in scope
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_method_already_defined() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::String);
        scope.define("len", GlossaType::Number); // Method name is already a defined variable in scope
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_genitive_method_call_empty() {
        let asm_stmt = AssembledStatement {
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_genitive_method_call(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_unwrap_empty() {
        let asm_stmt = AssembledStatement {
            unwraps: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_unwrap(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_subject_empty() {
        let asm_stmt = AssembledStatement {
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_subject(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_genitive_method_empty() {
        let asm_stmt = AssembledStatement {
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_genitive_method(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_nominatives_empty() {
        let asm_stmt = AssembledStatement {
            nominatives: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_nominatives(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_property_access(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_index_access_empty() {
        let asm_stmt = AssembledStatement {
            index_accesses: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_index_access(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_array_empty() {
        let asm_stmt = AssembledStatement {
            arrays: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_array(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_binary_op_empty() {
        let asm_stmt = AssembledStatement {
            operators: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_binary_op(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_object_empty() {
        let asm_stmt = AssembledStatement {
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_object(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_literal_empty() {
        let asm_stmt = AssembledStatement {
            literals: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_literal(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_object_fallback_empty() {
        let asm_stmt = AssembledStatement {
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_object_fallback(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_property_access_print_owner_not_in_scope() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγε".into(),
                normalized: "λέγε".into(),
                original: "λέγε".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            genitives: vec![Constituent {
                lemma: "owner".into(),
                normalized: "owner".into(),
                original: "owner".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            subject: Some(Constituent {
                lemma: "prop".into(),
                normalized: "prop".into(),
                original: "prop".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_property_access_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_property_access_print_owner_not_struct() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγε".into(),
                normalized: "λέγε".into(),
                original: "λέγε".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            genitives: vec![Constituent {
                lemma: "owner".into(),
                normalized: "owner".into(),
                original: "owner".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            subject: Some(Constituent {
                lemma: "prop".into(),
                normalized: "prop".into(),
                original: "prop".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("owner", GlossaType::Number); // Not a struct
        let result = classify_property_access_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_function_call_no_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἔστω".into(), // Binding verb
                normalized: "ἔστω".into(),
                original: "ἔστω".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            object: Some(Constituent {
                lemma: "myfunc".into(),
                normalized: "myfunc".into(),
                original: "myfunc".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: None,
                person: None,
            }),
            subject: None, // No subject
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define_function("myfunc", vec![], Some(GlossaType::Number));
        let result = classify_function_call(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_subjunctive_comparison_no_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἔστω".into(), // Binding verb
                normalized: "ἔστω".into(),
                original: "ἔστω".into(),
                person: None,
                number: None,
                tense: None,
                mood: Some(crate::morphology::Mood::Subjunctive),
                voice: None,
            }),
            operators: vec![crate::morphology::lexicon::BinaryOp::Eq],
            literals: vec![Literal::Number(5)],
            subject: None, // No subject
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_subjunctive_comparison(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_resolve_binding_target_subject_object_swap() {
        // Create a scope where 'subject_var' IS defined but 'object_var' is NOT defined.
        // This should trigger the Subject/Object swap logic.
        let mut scope = Scope::new();
        scope.define("subject_var", GlossaType::Number);

        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "subject_var".into(),
                normalized: "subject_var".into(),
                original: "subject_var".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            object: Some(Constituent {
                lemma: "object_var".into(),
                normalized: "object_var".into(),
                original: "object_var".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            ..Default::default()
        };

        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        // Since 'subject_var' was defined and 'object_var' was not, it should bind to 'object_var'
        assert_eq!(name, "object_var");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));

        // Ensure they were actually swapped
        assert_eq!(fixed_asm.subject.as_ref().unwrap().lemma, "object_var");
        assert_eq!(fixed_asm.object.as_ref().unwrap().lemma, "subject_var");
    }

    #[test]
    fn test_resolve_binding_target_subject_object_no_swap() {
        // Create a scope where NEITHER is defined.
        // This should skip the swap logic and fall into the 'else' branch,
        // binding to the subject and returning Cow::Borrowed.
        let scope = Scope::new();

        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "subject_var".into(),
                normalized: "subject_var".into(),
                original: "subject_var".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            object: Some(Constituent {
                lemma: "object_var".into(),
                normalized: "object_var".into(),
                original: "object_var".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            ..Default::default()
        };

        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "subject_var");
        assert!(matches!(fixed_asm, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_resolve_binding_target_no_subject_has_participle() {
        // This tests the "Fallback: Bind to first participle (if any remain)" case
        // We use a verb_lemma that exists in the lexicon so it's NOT treated as a "false participle"
        // Let's use "λεγω" which is definitely a verb
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "λεγω".into(),
                normalized: "λεγων".into(), // Actual participle
                original: "λέγων".into(),
                gender: crate::morphology::Gender::Masculine,
                case: crate::morphology::Case::Nominative,
                number: crate::morphology::Number::Singular,
                voice: crate::morphology::Voice::Active,
                tense: crate::morphology::Tense::Present,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "λεγων");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));
        assert!(fixed_asm.participles.is_empty()); // Should have been consumed
    }

    #[test]
    fn test_resolve_binding_target_false_participle() {
        // This tests the "false participle" check at the very beginning of the function
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "not_a_real_verb_lemma".into(), // Will fail lexicon lookup
                normalized: "false_participle".into(),
                original: "false_participle".into(),
                gender: crate::morphology::Gender::Masculine,
                case: crate::morphology::Case::Nominative,
                number: crate::morphology::Number::Singular,
                voice: crate::morphology::Voice::Active,
                tense: crate::morphology::Tense::Present,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "false_participle");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn test_resolve_binding_target_no_subject_no_participle() {
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Binding without subject")
        );
    }

    #[test]
    fn test_classify_query_containment_no_literal() {
        let asm_stmt = AssembledStatement {
            is_query: true,
            has_containment_preposition: true,
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![], // No literal element
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::List(Box::new(GlossaType::Number)));
        let result = classify_query(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        let stmt = result.unwrap();
        assert!(stmt.is_some());

        // Ensure the fallback literal generation (0) happened
        if let AnalyzedStatement::Query(exprs) = stmt.unwrap() {
            assert_eq!(exprs.len(), 1);
            if let AnalyzedExprKind::MethodCall { args, .. } = &exprs[0].expr {
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::UnaryOp { op, operand } = &args[0].expr {
                    assert_eq!(*op, crate::morphology::lexicon::UnaryOp::Ref);
                    assert!(matches!(operand.expr, AnalyzedExprKind::NumberLiteral(0)));
                } else {
                    panic!("Expected UnaryOp Ref");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Query");
        }
    }

    #[test]
    fn test_classify_insert_no_args() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τιθημι".into(), // insert verb
                normalized: "τιθημι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![],
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τιθημι", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "insert");
                assert!(args.is_empty());
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_insert_object() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τιθημι".into(), // insert verb
                normalized: "τιθημι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            object: Some(Constituent {
                lemma: "y".into(),
                normalized: "y".into(),
                original: "y".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: None,
                person: None,
            }),
            literals: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τιθημι", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "insert");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::Variable(var_name) = &args[0].expr {
                    assert_eq!(var_name, "y");
                } else {
                    panic!("Expected Variable argument");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_push_no_args() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ωθω".into(), // push verb
                normalized: "ωθω".into(),
                original: "ὠθεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![],
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("ωθω", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "push");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::NumberLiteral(val) = args[0].expr {
                    assert_eq!(val, 0); // fallback is 0
                } else {
                    panic!("Expected NumberLiteral fallback argument");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_expression_empty_exprs_propagate() {
        let scope = Scope::new();
        // Create an AssembledStatement that will produce an empty `exprs` array
        // but has `is_propagate` set to true.
        let asm_stmt = AssembledStatement {
            is_propagate: true,
            ..Default::default()
        };
        // No literals, operators, subject, object, or nested phrases -> exprs will be empty.

        let result = classify_expression(&asm_stmt, &scope);
        assert!(result.is_ok());

        if let AnalyzedStatement::Expression(exprs) = result.unwrap() {
            assert!(exprs.is_empty(), "Expected empty expressions array");
        } else {
            panic!("Expected AnalyzedStatement::Expression");
        }
    }
}
