import re

with open("src/semantic/conversion.rs", "r") as f:
    code = f.read()

search_block = """    if let Some(ref subj) = asm_stmt.subject
        && let Some(var_type) = scope.lookup(&subj.lemma)
    {
        args.insert(
            0,
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type.clone(),
            },
        );
    }

    if let Some(ref obj) = asm_stmt.object
        && let Some(var_type) = scope.lookup(&obj.lemma)
    {
        args.push(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
            glossa_type: var_type.clone(),
        });
    }"""

replace_block = """    if let Some(ref subj) = asm_stmt.subject {
        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else if crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() && subj.lemma != "self" && subj.lemma != "selfου" {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
    }

    if let Some(ref obj) = asm_stmt.object {
        if let Some(var_type) = scope.lookup(&obj.lemma) {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: var_type.clone(),
            });
        } else if crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() && obj.lemma != "self" && obj.lemma != "selfου" {
            return Err(GlossaError::undefined(obj.lemma.as_str()));
        }
    }"""

code = code.replace(search_block, replace_block)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(code)
