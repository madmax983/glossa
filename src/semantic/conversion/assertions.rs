use crate::errors::GlossaError;
use crate::semantic::assembler::state::AssembledStatement;
use crate::semantic::expressions::{build_binary_expr, literal_to_analyzed_expr};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
use crate::text::normalize_greek;

/// Helper: Detect δεῖ assertion pattern
/// Pattern: <condition> δεῖ (any word order)
/// Examples: "2 ἐν χ δεῖ", "δεῖ 2 ἐν χ"
pub fn classify_assertion(
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
pub fn classify_equality_assertion(
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

/// Helper: Detect subjunctive comparison (which looks like binding verb but isn't)
pub fn classify_subjunctive_comparison(
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
