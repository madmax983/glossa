use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::expressions::analyze_argument_expr;
use crate::semantic::model::*;
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;

use super::*;

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
