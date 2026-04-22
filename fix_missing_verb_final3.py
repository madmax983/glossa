import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    code = f.read()

search_block = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            // The memory said: "make sure to explicitly exempt valid verbless parser constructs (e.g., has_only_literals, is_operator_expr, valid is_match_arm configurations) to prevent internal compiler errors during codegen."
            // test_match_basic fails without it, but test_verbless_statement expects `ανθρωπος` to fail with MissingVerb.
            if subject.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }"""

replace_block = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if crate::morphology::lexicon::numeral_value(&subject.lemma).is_some() || subject.lemma == "αλλος" || subject.lemma == "αλλο" || subject.lemma == "μηδεν" {
                return Ok(());
            }
            return Err(AssemblyError::MissingVerb);
        }"""

code = code.replace(search_block, replace_block)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(code)

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
        } else if !crate::morphology::lexicon::is_err_word(&subj.lemma) {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
    }

    if let Some(ref obj) = asm_stmt.object {
        if let Some(var_type) = scope.lookup(&obj.lemma) {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: var_type.clone(),
            });
        } else if !crate::morphology::lexicon::is_err_word(&obj.lemma) {
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

code = code.replace(search_block_conv, replace_block_conv)

search_block_conv2 = """            if !scope.is_defined(&subj.lemma) && !crate::morphology::lexicon::is_err_word(&subj.lemma) {
                // Ignore undefined errors for "self" or "selfου" inside structs/traits
                if subj.lemma != "self" && subj.lemma != "selfου" {
                    return Err(GlossaError::undefined(subj.lemma.as_str()));
                }
            }"""

replace_block_conv2 = """            if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                // Ignore undefined errors for "self" or "selfου" inside structs/traits
                if subj.lemma != "self" && subj.lemma != "selfου" {
                    return Err(GlossaError::undefined(subj.lemma.as_str()));
                }
            }"""

code = code.replace(search_block_conv2, replace_block_conv2)

search_block_conv3 = """            if !scope.is_defined(&obj.lemma) && !crate::morphology::lexicon::is_err_word(&obj.lemma) {
                if obj.lemma != "self" && obj.lemma != "selfου" {
                    return Err(GlossaError::undefined(obj.lemma.as_str()));
                }
            }"""

replace_block_conv3 = """            if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() {
                if obj.lemma != "self" && obj.lemma != "selfου" {
                    return Err(GlossaError::undefined(obj.lemma.as_str()));
                }
            }"""

code = code.replace(search_block_conv3, replace_block_conv3)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(code)
