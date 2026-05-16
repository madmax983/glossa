use crate::semantic::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr, literal_to_type,
};
use crate::semantic::patterns::detect_iterator_pattern;
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::morphology::{self};
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;
use crate::semantic::{Constituent, Literal};

pub(crate) use crate::semantic::conversion::values::*;

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
pub(crate) fn classify_iterator_pattern(
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
pub(crate) fn classify_property_access_print(
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
pub(crate) fn classify_function_call(
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

    // ⚡ Bolt Optimization: Pre-allocate vector capacity to avoid intermediate `.collect()` and reallocations
    let mut args = Vec::with_capacity(asm_stmt.literals.len() + asm_stmt.nested_phrases.len());

    args.extend(asm_stmt.literals.iter().map(literal_to_analyzed_expr));

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
pub(crate) fn resolve_function_name(
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
pub(crate) fn classify_subjunctive_comparison(
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
pub(crate) fn resolve_binding_target<'a>(
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
pub(crate) fn classify_variable_binding(
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
pub(crate) fn classify_assignment(
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
pub(crate) fn classify_collection_mutation(
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

pub(crate) fn classify_pop(
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

pub(crate) fn classify_push(
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

pub(crate) fn classify_insert(
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
pub(crate) fn classify_assertion(
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
pub(crate) fn classify_equality_assertion(
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

pub(crate) fn try_print_binary_op(
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

pub(crate) fn try_print_property_access(
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

pub(crate) fn try_print_index_access(
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

pub(crate) fn try_print_unwrap(
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

pub(crate) fn try_print_default(
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
pub(crate) fn classify_print(
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
pub(crate) fn classify_query(
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
pub(crate) fn classify_containment_query(
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
pub(crate) fn classify_expression(
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
pub(crate) fn try_parse_genitive_method_call(
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

    let args: Vec<AnalyzedExpr> = asm_stmt
        .literals
        .iter()
        .map(literal_to_analyzed_expr)
        .collect();

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
pub(crate) fn classify_genitive_method_call(
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
pub(crate) fn detect_enum_variant(
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

pub(crate) fn extract_unwrap(
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

pub(crate) fn extract_enum_from_subject(
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

pub(crate) fn extract_genitive_method(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(result) = try_parse_genitive_method_call(asm_stmt, scope) {
        return Ok(Some(result));
    }
    Ok(None)
}

pub(crate) fn extract_enum_from_nominatives(
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

pub(crate) fn extract_property_access(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    let Some((owner, method)) = asm_stmt.property_accesses.first() else {
        return Ok(None);
    };

    let receiver = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(owner.clone().into()),
        glossa_type: GlossaType::Unknown,
    };
    Ok(Some((
        AnalyzedExpr {
            expr: AnalyzedExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: method.clone().into(),
                args: vec![],
            },
            glossa_type: GlossaType::Number,
        },
        GlossaType::Number,
    )))
}

pub(crate) fn extract_index_access(
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

pub(crate) fn extract_array(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    let Some(array_elements) = asm_stmt.arrays.first() else {
        return Ok(None);
    };

    let analyzed_elements: Vec<AnalyzedExpr> = array_elements
        .iter()
        .map(|e| analyze_argument_expr(e, scope))
        .collect::<Result<Vec<_>, _>>()?;

    let element_type = analyzed_elements
        .first()
        .map(|e| e.glossa_type.clone())
        .unwrap_or(GlossaType::Unknown);

    Ok(Some((
        AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(analyzed_elements),
            glossa_type: GlossaType::List(Box::new(element_type)),
        },
        GlossaType::List(Box::new(GlossaType::Unknown)),
    )))
}

pub(crate) fn extract_binary_op(
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

pub(crate) fn extract_literal(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(lit) = asm_stmt.literals.first() {
        return Ok(Some((literal_to_analyzed_expr(lit), literal_to_type(lit))));
    }
    Ok(None)
}

pub(crate) fn extract_enum_from_object(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref obj) = asm_stmt.object {
        return Ok(detect_enum_variant(obj, &asm_stmt.literals));
    }
    Ok(None)
}

pub(crate) fn extract_object_fallback(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    let Some(ref obj) = asm_stmt.object else {
        return Ok(None);
    };

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

    Ok(Some((
        AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
            glossa_type: GlossaType::Unknown,
        },
        GlossaType::Unknown,
    )))
}
