use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::expressions::literal_to_analyzed_expr;
use crate::semantic::model::AnalyzedStatement;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind};
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;
use crate::semantic::{Constituent, Literal};
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
