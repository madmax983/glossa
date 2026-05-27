use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::model::*;
use crate::semantic::resolver::Scope;

use super::*;

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
