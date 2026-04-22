import re

with open("src/semantic/conversion.rs", "r") as f:
    code = f.read()

search_block_conv = """    if let Some(ref subj) = asm_stmt.subject {
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

replace_block_conv = """    if let Some(ref subj) = asm_stmt.subject {
        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else if crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() && subj.lemma != "self" && subj.lemma != "selfου" {
            // Memory states: explicitly exempt the keywords "self" and "selfου" in extract_property_access and try_print_property_access
            // Wait, this is try_print_default. If it's a test failing like trait_tests, maybe variables aren't strictly defined in those tests because of God tests?
            // Actually, `scope.is_defined` might fail if `try_print_default` is given random un-bound things. Let's revert this file to original and only fix `extract_value`?
            // Actually, we must be careful with undefined checks in arbitrary places. Let's check the test failure: `UndefinedName { name: "ξ" }`
        }
    }"""

# Reverting conversion.rs entirely because we broke trait tests that rely on undefined variables or use different binding rules
