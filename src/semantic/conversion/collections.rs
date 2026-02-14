use crate::errors::GlossaError;
use crate::semantic::assembler::state::AssembledStatement;
use crate::semantic::expressions::literal_to_analyzed_expr;
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
use crate::text::normalize_greek;

/// Helper: Detect collection mutation (pop, push, insert)
pub fn classify_collection_mutation(
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
