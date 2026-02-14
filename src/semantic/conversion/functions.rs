use super::expressions::try_parse_genitive_method_call;
use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::semantic::assembler::state::AssembledStatement;
use crate::semantic::expressions::{analyze_argument_expr, literal_to_analyzed_expr};
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};
use crate::text::normalize_greek;

/// Helper: Detect user-defined function call
pub fn classify_function_call(
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

/// Helper: Detect genitive method call (owner.method)
pub fn classify_genitive_method_call(
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
