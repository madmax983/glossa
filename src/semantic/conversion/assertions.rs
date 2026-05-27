use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::*;
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;

use super::*;

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
