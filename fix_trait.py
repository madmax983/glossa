import re

with open("src/semantic/conversion.rs", "r") as f:
    content = f.read()

new_func = """fn try_print_default(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Vec<AnalyzedExpr>, GlossaError> {
    let mut args =
        build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators)?;

    if let Some(ref subj) = asm_stmt.subject {
        if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() && subj.lemma != "αληθες" && subj.lemma != "ψευδος" && subj.lemma != "self" && subj.lemma != "selfου" {
            // Check if it's a property lookup or function call
            if asm_stmt.genitives.is_empty() && !scope.is_function(&subj.lemma) && scope.lookup_type(&subj.lemma).is_none() {
                // Return unknown if in trait context? No, we shouldn't fail.
                // It looks like `try_print_default` is catching property accesses that evaluate as Subject!
                // Let's just restore the original behavior for `try_print_default` and `extract_object_fallback` where it DOES NOT throw UndefinedName.
                // We're breaking trait methods.
                // Instead, where SHOULD we throw UndefinedName?
                // Wait, if we just remove the `&& name != "ἀληθές"` stuff and the UndefinedName check entirely,
                // we break the test `test_undefined_variable_evaluates_to_zero_silently`.
                // Can we just explicitly check if `name == "ἄγνωστος"`? NO! That's cheating.
                // How about we just don't throw UndefinedName if it's inside a trait impl?
                // `Scope` has `trait_impls` or we can check `scope.is_defined(self)`.
                if scope.is_defined("self") {
                    // Inside trait impl
                } else {
                    return Err(GlossaError::undefined(&*subj.original));
                }
            }
        }
        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else {
             args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: crate::semantic::GlossaType::Unknown,
                },
            );
        }
    } else if let Some(ref obj) = asm_stmt.object {
         if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() && obj.lemma != "αληθες" && obj.lemma != "ψευδος" && obj.lemma != "self" && obj.lemma != "selfου" {
            if asm_stmt.genitives.is_empty() && !scope.is_function(&obj.lemma) && scope.lookup_type(&obj.lemma).is_none() {
                if !scope.is_defined("self") {
                    return Err(GlossaError::undefined(&*obj.original));
                }
            }
        }
    }

    Ok(args)
}"""

content = re.sub(
    r'fn try_print_default\(\n    asm_stmt: &AssembledStatement,\n    scope: &mut Scope,\n\) -> Result<Vec<AnalyzedExpr>, GlossaError> \{.*?\n    Ok\(args\)\n\}',
    new_func,
    content,
    flags=re.DOTALL
)

new_extract = """    /// Fallback for extracting standalone terms like literals or unknown variables
    fn extract_object_fallback(
        &self,
        stmt: &AssembledStatement,
        scope: &mut Scope,
    ) -> Result<AnalyzedStatement, GlossaError> {
        if let Some(ref obj) = stmt.object {
            if obj.part_of_speech == crate::morphology::PartOfSpeech::Noun {
                let name = obj.original.to_string();
                if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() && name != "ἀληθές" && name != "ψεῦδος" && obj.lemma != "self" && obj.lemma != "selfου" {
                    if stmt.genitives.is_empty() && stmt.property_accesses.is_empty() && !scope.is_function(&obj.lemma) && scope.lookup_type(&obj.lemma).is_none() {
                        if !scope.is_defined("self") {
                            return Err(GlossaError::undefined(&name));
                        }
                    }
                }
            }
        }

        if let Some(ref subj) = stmt.subject {
            if subj.part_of_speech == crate::morphology::PartOfSpeech::Noun {
                let name = subj.original.to_string();
                if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() && name != "ἀληθές" && name != "ψεῦδος" && subj.lemma != "self" && subj.lemma != "selfου" {
                    if stmt.genitives.is_empty() && stmt.property_accesses.is_empty() && !scope.is_function(&subj.lemma) && scope.lookup_type(&subj.lemma).is_none() {
                        if !scope.is_defined("self") {
                            return Err(GlossaError::undefined(&name));
                        }
                    }
                }
            }
        }"""

content = re.sub(
    r'    /// Fallback for extracting standalone terms like literals or unknown variables\n    fn extract_object_fallback\(\n        &self,\n        stmt: &AssembledStatement,\n        _scope: &mut Scope,\n    \) -> Result<AnalyzedStatement, GlossaError> \{.*?\n        \}\]\}',
    new_extract,
    content,
    flags=re.DOTALL
)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(content)
