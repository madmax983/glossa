import re

content = open("src/semantic/conversion.rs").read()

new_helper = """
fn is_resolved(lemma: &str, normalized: &str, asm_stmt: &AssembledStatement, scope: &Scope) -> bool {
    let is_self_property_access = asm_stmt
        .property_accesses
        .iter()
        .any(|(owner, _)| owner == lemma || owner == normalized);
    let is_in_trait_or_struct = !asm_stmt.nested_phrases.is_empty()
        || scope.lookup("self").is_some()
        || scope.is_defined("self")
        || scope.lookup_binding("self").is_some();

    scope.lookup(lemma).is_some()
        || is_self_property_access
        || is_in_trait_or_struct
        || scope.lookup_binding(lemma).is_some()
        || scope.is_defined(lemma)
}

fn try_print_default(
"""

content = content.replace("fn try_print_default(\n", new_helper)

old_try_print_logic = """    if let Some(ref subj) = asm_stmt.subject {
        let is_self_property_access = asm_stmt
            .property_accesses
            .iter()
            .any(|(owner, _)| owner == subj.lemma || owner == subj.normalized);
        let is_in_trait_or_struct = !asm_stmt.nested_phrases.is_empty()
            || scope.lookup("self").is_some()
            || scope.is_defined("self")
            || scope.lookup_binding("self").is_some();
        let is_resolved = scope.lookup(&subj.lemma).is_some()
            || is_self_property_access
            || is_in_trait_or_struct
            || scope.lookup_binding(&subj.lemma).is_some()
            || scope.is_defined(&subj.lemma);

        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else if !is_resolved {
            return Err(GlossaError::undefined(subj.lemma.clone()));
        } else {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: scope
                        .lookup_binding(&subj.lemma)
                        .map(|b| b.glossa_type.clone())
                        .unwrap_or(GlossaType::Unknown),
                },
            );
        }
    }

    if let Some(ref obj) = asm_stmt.object {
        let is_self_property_access = asm_stmt
            .property_accesses
            .iter()
            .any(|(owner, _)| owner == obj.lemma || owner == obj.normalized);
        let is_in_trait_or_struct = !asm_stmt.nested_phrases.is_empty()
            || scope.lookup("self").is_some()
            || scope.is_defined("self")
            || scope.lookup_binding("self").is_some();
        let is_resolved = scope.lookup(&obj.lemma).is_some()
            || is_self_property_access
            || is_in_trait_or_struct
            || scope.lookup_binding(&obj.lemma).is_some()
            || scope.is_defined(&obj.lemma);

        if let Some(var_type) = scope.lookup(&obj.lemma) {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: var_type.clone(),
            });
        } else if !is_resolved {
            return Err(GlossaError::undefined(obj.lemma.clone()));
        } else {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: scope
                    .lookup_binding(&obj.lemma)
                    .map(|b| b.glossa_type.clone())
                    .unwrap_or(GlossaType::Unknown),
            });
        }
    }"""

new_try_print_logic = """    if let Some(ref subj) = asm_stmt.subject {
        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else if !is_resolved(&subj.lemma, &subj.normalized, asm_stmt, scope) {
            return Err(GlossaError::undefined(subj.lemma.clone()));
        } else {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: scope
                        .lookup_binding(&subj.lemma)
                        .map(|b| b.glossa_type.clone())
                        .unwrap_or(GlossaType::Unknown),
                },
            );
        }
    }

    if let Some(ref obj) = asm_stmt.object {
        if let Some(var_type) = scope.lookup(&obj.lemma) {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: var_type.clone(),
            });
        } else if !is_resolved(&obj.lemma, &obj.normalized, asm_stmt, scope) {
            return Err(GlossaError::undefined(obj.lemma.clone()));
        } else {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: scope
                    .lookup_binding(&obj.lemma)
                    .map(|b| b.glossa_type.clone())
                    .unwrap_or(GlossaType::Unknown),
            });
        }
    }"""
content = content.replace(old_try_print_logic, new_try_print_logic)
open("src/semantic/conversion.rs", "w").write(content)
