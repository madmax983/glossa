use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::conversion::shared::*;
use crate::semantic::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr, literal_to_type,
};
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind};
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;

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
    _scope: &Scope,
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
pub(crate) fn extract_value(
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
#[cfg(test)]
mod tests {
    use crate::semantic::conversion::classification::*;
    use crate::semantic::{AnalyzedStatement, Constituent, Literal};

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
    fn test_resolve_binding_target_no_subject_has_participle() {
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "participle".into(),
                normalized: "participle".into(),
                original: "participle".into(),
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
        assert_eq!(result.unwrap().0, "participle");
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
        // Create an AssembledStatement that will produce an empty `exprs` array
        // but has `is_propagate` set to true.
        let asm_stmt = AssembledStatement {
            is_propagate: true,
            ..Default::default()
        };
        // No literals, operators, subject, object, or nested phrases -> exprs will be empty.

        let result = classify_expression(&asm_stmt);
        assert!(result.is_ok());

        if let AnalyzedStatement::Expression(exprs) = result.unwrap() {
            assert!(exprs.is_empty(), "Expected empty expressions array");
        } else {
            panic!("Expected AnalyzedStatement::Expression");
        }
    }
}
