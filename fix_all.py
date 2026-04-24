import re

# 1. Fix Undefined Variable logic
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
            return Err(GlossaError::undefined(&*subj.original));
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
            return Err(GlossaError::undefined(&*obj.original));
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
                if !scope.is_defined(&name) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() && name != "ἀληθές" && name != "ψεῦδος" {
                    return Err(GlossaError::undefined(&name));
                }
            }
        }

        if let Some(ref subj) = stmt.subject {
            if subj.part_of_speech == crate::morphology::PartOfSpeech::Noun {
                let name = subj.original.to_string();
                if !scope.is_defined(&name) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() && name != "ἀληθές" && name != "ψεῦδος" {
                    return Err(GlossaError::undefined(&name));
                }
            }
        }"""

content = re.sub(
    r'    /// Fallback for extracting standalone terms like literals or unknown variables\n    fn extract_object_fallback\(\n        &self,\n        stmt: &AssembledStatement,\n        _scope: &mut Scope,\n    \) -> Result<AnalyzedStatement, GlossaError> \{',
    new_extract,
    content
)
with open("src/semantic/conversion.rs", "w") as f:
    f.write(content)


# 2. Fix missing verb logic
with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

# First replace `&& subject.lemma == "ανθρωπος"` with `return Err(AssemblyError::MissingVerb);`
new_func = """    fn check_missing_verb(&self, ctx: &StatementContext) -> Result<(), AssemblyError> {
        if ctx.has_only_literals
            || ctx.is_operator_expr
            || ctx.is_propagate
            || ctx.is_string_method
            || ctx.is_property_access
            || ctx.is_index_access
            || ctx.is_nested_phrase
            || ctx.is_block
            || ctx.is_unwrap
            || ctx.is_genitive_possession
            || ctx.is_multiple_nominatives
            || ctx.is_array
            || ctx.has_delimiter
        {
            return Ok(());
        }
        if (!self.state.literals.is_empty()
            || !self.state.index_accesses.is_empty()
            || !self.state.property_accesses.is_empty())
            && self.state.subject.is_none()
            && self.state.object.is_none()
        {
            return Ok(());
        }

        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if crate::morphology::lexicon::numeral_value(&subject.lemma).is_none()
                && subject.lemma != "αλλο"
                && subject.lemma != "μηδεν"
                && subject.lemma != "ουδεν"
                && subject.lemma != "τι"
                && subject.lemma != "εν"
                && self.state.verb.is_none()
            {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }

        if self.state.verb.is_some() {
            return Ok(());
        }

        Err(AssemblyError::MissingVerb)
    }"""
content = re.sub(
    r'    fn check_missing_verb\(&self, ctx: &StatementContext\) -> Result<\(\), AssemblyError> \{.*?\n    \}',
    new_func,
    content,
    flags=re.DOTALL
)

# And fix DoubleSubject check
content = re.sub(
    r'&&\s*!crate::morphology::lexicon::is_print_verb\(&verb\.lemma\)\s*&&\s*!crate::morphology::lexicon::is_find_verb\(&verb\.lemma\)',
    '',
    content
)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(content)


# 3. Bypass MissingVerb check for single subject variable inside control_flow
with open("src/semantic/control_flow.rs", "r") as f:
    content = f.read()

new_func = """fn skip_first_word_and_parse(
    clause: &Clause,
    scope: &mut Scope,
) -> Result<AnalyzedExpr, GlossaError> {
    // Create a modified clause without the first word
    let mut modified_clause = clause.clone();

    // Remove the first word from the first expression
    if let Some(first_expr) = modified_clause.expressions.first_mut()
        && let Expr::Phrase(terms) = first_expr
        && !terms.is_empty()
    {
        terms.remove(0);

        if modified_clause.expressions.len() == 1 {
            if let Expr::Phrase(terms) = &modified_clause.expressions[0] {
                if terms.len() == 1 {
                    let lemma = crate::morphology::lexicon::get_lemma(&terms[0]);

                    if let Some(val) = crate::morphology::lexicon::numeral_value(&lemma) {
                         return Ok(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(val),
                            glossa_type: crate::semantic::GlossaType::Number,
                        });
                    }

                    if lemma == "αλλο" {
                         return Ok(AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(true),
                            glossa_type: crate::semantic::GlossaType::Boolean,
                        });
                    }

                    if lemma == "μηδεν" || lemma == "ουδεν" || lemma == "τι" || lemma == "εν" {
                         return Ok(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(lemma.into()),
                            glossa_type: crate::semantic::GlossaType::Unknown,
                        });
                    }

                    let glossa_type = scope.lookup(&lemma).cloned().unwrap_or(crate::semantic::GlossaType::Unknown);
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(lemma.into()),
                        glossa_type,
                    });
                }
            }
        }
    }

    // Parse the modified clause as a statement
    let stmt = Statement::Regular {
        clauses: vec![modified_clause],
        is_query: false,
        is_propagate: false,
    };

    // We bypass the top-level MissingVerb check by extracting the expression manually
    // from the assembled statement if it's a simple fallback condition.
    let analyzed_res = assemble_statement(&stmt);
    let analyzed = match analyzed_res {
        Ok(a) => a,
        Err(crate::errors::AssemblyError::MissingVerb) => {
            return Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb));
        }
        Err(e) => return Err(GlossaError::AssemblyError(e)),
    };

    let converted =
        match crate::semantic::conversion::convert_assembled_to_analyzed(&analyzed, scope) {
            Ok(c) => c,
            Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb)) => {
                if let Some(ref subj) = analyzed.subject {
                    let var_expr = crate::semantic::AnalyzedExpr {
                        expr: crate::semantic::AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: crate::semantic::GlossaType::Unknown,
                    };
                    AnalyzedStatement::Expression(vec![var_expr])
                } else if let Some(ref obj) = analyzed.object {
                    let var_expr = crate::semantic::AnalyzedExpr {
                        expr: crate::semantic::AnalyzedExprKind::Variable(obj.lemma.clone()),
                        glossa_type: crate::semantic::GlossaType::Unknown,
                    };
                    AnalyzedStatement::Expression(vec![var_expr])
                } else {
                    return Err(GlossaError::semantic("Empty condition in conditional"));
                }
            }
            Err(e) => return Err(e),
        };

    match converted {
        AnalyzedStatement::Expression(exprs) => {
            if exprs.is_empty() {
                Err(GlossaError::semantic("Empty expression in clause"))
            } else {
                Ok(exprs[0].clone())
            }
        }
        _ => Err(GlossaError::semantic("Expected expression, found statement")),
    }
}"""
content = re.sub(
    r'fn skip_first_word_and_parse\(\n    clause: &Clause,\n    scope: &mut Scope,\n\) -> Result<AnalyzedExpr, GlossaError> \{.*?\n        _ => Err\(GlossaError::semantic\("Expected expression, found statement"\)\),\n    \}\n\}',
    new_func,
    content,
    flags=re.DOTALL
)

new_func = """fn parse_match_pattern(expr: &Expr, scope: &mut Scope) -> Result<AnalyzedExpr, GlossaError> {
    // Pattern is typically: value ᾖ
    if let Expr::Phrase(terms) = expr {
        if terms.is_empty() {
            return Err(GlossaError::semantic("Empty match pattern"));
        }

        // Get first word (the pattern value)
        if let Expr::Word(w) = &terms[0] {
            let normalized = &w.normalized;

            // Check if it's ἄλλο (wildcard)
            if normalized == "αλλο" {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                });
            }

            // Check if it's a numeral
            if let Some(val) = lexicon::numeral_value(normalized) {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(val),
                    glossa_type: GlossaType::Number,
                });
            }

            // Also check 'μηδὲν', 'οὐδὲν' etc
            if normalized == "μηδεν" || normalized == "ουδεν" || normalized == "τι" || normalized == "εν" {
                return Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(normalized.clone()),
                    glossa_type: GlossaType::Unknown,
                });
            }

            // Otherwise, treat as variable reference
            let var_type = scope
                .lookup(normalized)
                .cloned()
                .unwrap_or(GlossaType::Unknown);
            // Wait, we don't throw an error here, `parse_match_pattern` might just parse literals
            // But we actually DO throw an error if it's not defined!
            // BUT, `μηδὲν ᾖ` the `μηδὲν` is NOT defined in the scope, it's just a pattern token!
            // Wait, if it's a variable reference, then it IS defined in the scope.
            // Let's just leave the rest of parse_match_pattern as it was:
            if var_type == GlossaType::Unknown && !scope.is_function(normalized) {
                // Actually, if it's not a function and not defined, let's just return it as Unknown instead of erroring out right here.
                // It might be evaluated correctly later or it's a pattern we couldn't resolve perfectly.
                // No, we should throw an error if we enforce strict scoping, but `parse_match_pattern` might be checking literal variables that shouldn't error.
            }
            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(normalized.clone()),
                glossa_type: var_type,
            });
        }
    } else if let Expr::Word(w) = expr {"""

content = re.sub(
    r'fn parse_match_pattern\(expr: &Expr, scope: &mut Scope\) -> Result<AnalyzedExpr, GlossaError> \{.*?\n    \} else if let Expr::Word\(w\) = expr \{',
    new_func,
    content,
    flags=re.DOTALL
)

with open("src/semantic/control_flow.rs", "w") as f:
    f.write(content)
