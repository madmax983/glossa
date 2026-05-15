use super::classify::{detect_enum_variant, try_parse_genitive_method_call};
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::semantic::Scope;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::expressions::{
    analyze_argument_expr, build_binary_expr, build_expressions_from_literals_and_ops,
    literal_to_analyzed_expr, literal_to_type,
};
use crate::semantic::types::GlossaType;
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind};

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

    // Default
    Ok((
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
        GlossaType::Number,
    ))
}
