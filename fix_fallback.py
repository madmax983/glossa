import re

with open("src/semantic/conversion.rs", "r") as f:
    code = f.read()

search_block = """    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }"""

replace_block = """    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                // Ignore undefined errors for "self" or "selfου" inside structs/traits
                if subj.lemma != "self" && subj.lemma != "selfου" {
                    return Err(GlossaError::undefined(subj.lemma.as_str()));
                }
            }
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() {
                if obj.lemma != "self" && obj.lemma != "selfου" {
                    return Err(GlossaError::undefined(obj.lemma.as_str()));
                }
            }
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }"""

code = code.replace(search_block, replace_block)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(code)
