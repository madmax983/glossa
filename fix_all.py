import sys

def main():
    # 1. src/semantic/assembly/mod.rs
    content = open("src/semantic/assembly/mod.rs").read()
    old_code = """        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }"""

    new_code = """        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            // We evaluate all verbs uniformly (no bypass for print/find verbs).
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }"""
    content = content.replace(old_code, new_code)
    with open("src/semantic/assembly/mod.rs", "w") as f:
        f.write(content)

    # 2. src/semantic/conversion.rs
    content = open("src/semantic/conversion.rs").read()

    old_try_print = """    if let Some(ref subj) = asm_stmt.subject
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

    new_try_print = """    if let Some(ref subj) = asm_stmt.subject {
        if let Some(var_type) = scope.lookup(&subj.lemma) {
            args.insert(
                0,
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                    glossa_type: var_type.clone(),
                },
            );
        } else {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
    }

    if let Some(ref obj) = asm_stmt.object {
        if let Some(var_type) = scope.lookup(&obj.lemma) {
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: var_type.clone(),
            });
        } else {
            return Err(GlossaError::undefined(obj.lemma.as_str()));
        }
    }"""
    content = content.replace(old_try_print, new_try_print)

    old_obj = """        if !scope.is_defined(obj_lemma) {
            return Err(GlossaError::undefined(obj_lemma.as_str()));
        }"""

    new_obj = """        if !scope.is_defined(obj_lemma) {
            // Function definitions might have an undefined object as the name
            if let Some(verb) = &asm_stmt.verb {
                if verb.lemma == "ὁρίζω" {
                    return Ok(Some((
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                            glossa_type: GlossaType::Unknown,
                        },
                        GlossaType::Unknown,
                    )));
                }
            }
            if !asm_stmt.genitives.is_empty() {
                // If it's used with a genitive property owner, skip undefined check for the property name
                return Ok(Some((
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                        glossa_type: GlossaType::Unknown,
                    },
                    GlossaType::Unknown,
                )));
            }
            return Err(GlossaError::undefined(obj_lemma.as_str()));
        }"""
    content = content.replace(old_obj, new_obj)

    old_expr = """    // Fallback: If no literals/ops, check Subject/Object
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

    new_expr = """    // We must check DoubleSubject here before proceeding to Variable checks.
    // If the Assembler successfully produced a DoubleSubject error previously,
    // we would have caught it there, but sometimes DoubleSubject bypasses Assembler validations
    // or isn't caught. In `ὁ ἄνθρωπος ὁ θεὸς λέγει`, we have 2 nominatives!
    if asm_stmt.subject.is_some() && !asm_stmt.nominatives.is_empty() && asm_stmt.operators.is_empty() {
        let is_function_call = !asm_stmt.nested_phrases.is_empty()
            || !asm_stmt.blocks.is_empty()
            || !asm_stmt.literals.is_empty();
        let is_special_pattern =
            !asm_stmt.property_accesses.is_empty() || asm_stmt.is_query;

        let verb_lemma = asm_stmt.verb.as_ref().map(|v| v.lemma.as_str()).unwrap_or("");

        let is_binding_verb = crate::morphology::lexicon::is_binding_verb(verb_lemma);
        let is_print_verb = crate::morphology::lexicon::is_print_verb(verb_lemma);
        let is_find_verb = crate::morphology::lexicon::is_find_verb(verb_lemma);
        let is_push_pop = verb_lemma == "τίθημι" || verb_lemma == "αἴρω";

        if !is_function_call && !is_special_pattern && !is_binding_verb && !is_print_verb && !is_find_verb && !is_push_pop {
             return Err(GlossaError::AssemblyError(crate::errors::AssemblyError::DoubleSubject));
        }
    }

    // Check undefined variables upfront if we have operators (since we might build them as variables)
    if !asm_stmt.operators.is_empty() && asm_stmt.literals.len() < 2 {
        #[allow(clippy::collapsible_if)]
        if let Some(ref subj) = asm_stmt.subject {
            if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                // If it's a field being returned/assigned (e.g. selfου ξ), it might not be explicitly in scope.
                if asm_stmt.property_accesses.is_empty() && asm_stmt.genitives.is_empty() && subj.lemma != "self" && subj.lemma != "selfου" {
                    return Err(GlossaError::undefined(subj.lemma.as_str()));
                }
            }
        }
        #[allow(clippy::collapsible_if)]
        if let Some(ref obj) = asm_stmt.object {
            if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() {
                if asm_stmt.property_accesses.is_empty() && asm_stmt.genitives.is_empty() && obj.lemma != "self" && obj.lemma != "selfου" {
                    return Err(GlossaError::undefined(obj.lemma.as_str()));
                }
            }
        }
    }

    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            if asm_stmt.verb.is_none() && asm_stmt.operators.is_empty() && !asm_stmt.is_propagate {
                // If we have a standalone subject and no verb at all, and no operators/propagation, it's a Missing Verb error
                // Exception: Numerals are valid standalone expressions (e.g., list length access)
                if crate::morphology::lexicon::numeral_value(&subj.lemma).is_none()
                   && !["ἄλλο", "μηδέν", "οὐδέν", "τι", "ἕν"].contains(&subj.lemma.as_str()) {
                    // One last exception: if it's part of a trait impl block or property access, it might just be an expression
                    // We'll let extract_property_access handle undefined variables correctly if it's a property.
                    if asm_stmt.property_accesses.is_empty() && asm_stmt.genitives.is_empty() {
                        return Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb));
                    }
                }
            }

            if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                // Ignore undefined subject if this is part of a function definition or property access
                let is_func_def = asm_stmt.verb.as_ref().map_or(false, |v| v.lemma == "ὁρίζω");
                if !is_func_def && asm_stmt.property_accesses.is_empty() && asm_stmt.genitives.is_empty() {
                    return Err(GlossaError::undefined(subj.lemma.as_str()));
                }
            }

            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() {
                if asm_stmt.property_accesses.is_empty() && asm_stmt.genitives.is_empty() {
                    return Err(GlossaError::undefined(obj.lemma.as_str()));
                }
            }

            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }"""
    content = content.replace(old_expr, new_expr)

    old_fn = """fn try_print_property_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Option<Vec<AnalyzedExpr>> {
    if !asm_stmt.property_accesses.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.property_accesses.len());
        for (owner, method) in &asm_stmt.property_accesses {
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(owner.clone().into()),
                glossa_type: scope.lookup(owner).cloned().unwrap_or(GlossaType::Unknown),
            };"""

    new_fn = """fn try_print_property_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<Vec<AnalyzedExpr>>, GlossaError> {
    if !asm_stmt.property_accesses.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.property_accesses.len());
        for (owner, method) in &asm_stmt.property_accesses {
            if !scope.is_defined(owner) && owner != "self" && owner != "selfου" {
                return Err(GlossaError::undefined(owner));
            }
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(owner.clone().into()),
                glossa_type: scope.lookup(owner).cloned().unwrap_or(GlossaType::Unknown),
            };"""
    content = content.replace(old_fn, new_fn)

    old_ret = """        }
        return Some(args);
    }
    None
}"""
    new_ret = """        }
        return Ok(Some(args));
    }
    Ok(None)
}"""
    content = content.replace(old_ret, new_ret)

    old_call = """            if let Some(args) = try_print_property_access(asm_stmt, scope) {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }"""
    new_call = """            if let Some(args) = try_print_property_access(asm_stmt, scope)? {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }"""
    content = content.replace(old_call, new_call)

    old_test = """    #[test]
    fn test_try_print_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_property_access(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }"""
    new_test = """    #[test]
    fn test_try_print_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_property_access(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }"""
    content = content.replace(old_test, new_test)

    old_extract = """fn extract_property_access(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some((owner, method)) = asm_stmt.property_accesses.first() {
        let receiver = AnalyzedExpr {"""

    new_extract = """fn extract_property_access(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some((owner, method)) = asm_stmt.property_accesses.first() {
        if !scope.is_defined(owner) && owner != "self" && owner != "selfου" {
             return Err(GlossaError::undefined(owner));
        }
        let receiver = AnalyzedExpr {"""
    content = content.replace(old_extract, new_extract)

    old_genitive = """    let owner_type = scope.lookup(owner_lemma)?;"""
    new_genitive = """    let owner_type = if owner_lemma == "self" || owner_lemma == "selfου" {
        GlossaType::Unknown
    } else {
        scope.lookup(owner_lemma)?.clone()
    };"""
    content = content.replace(old_genitive, new_genitive)

    old_owner = """    // Check if owner is a struct type in scope
    let Some(owner_type) = scope.lookup(owner_lemma) else {
        return Ok(None);
    };"""
    new_owner = """    // Check if owner is a struct type in scope
    let Some(owner_type) = (if owner_lemma == "self" || owner_lemma == "selfου" {
        Some(&GlossaType::Unknown)
    } else {
        scope.lookup(owner_lemma)
    }) else {
        return Ok(None);
    };"""
    content = content.replace(old_owner, new_owner)

    with open("src/semantic/conversion.rs", "w") as f:
        f.write(content)

    # 3. src/semantic/resolver.rs
    content = open("src/semantic/resolver.rs").read()
    old_code = """    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }"""

    new_code = """    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some() || self.is_function(name)
    }"""
    content = content.replace(old_code, new_code)
    with open("src/semantic/resolver.rs", "w") as f:
        f.write(content)

    # 4. tests/havoc_issue_echo.rs
    content = open("tests/havoc_issue_echo.rs").read()
    old_test_1 = """#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let _ = analyze_program(&ast).unwrap();
    // Havoc constraints: "Never write 'Happy Path' tests. If it works, you failed."
    // In Echo bug, double subject compiles with zero errors instead of failing gracefully.
}"""

    new_test_1 = """#[test]
fn test_double_subject_should_pass_havoc_constraint() {
    let source = "ὁ ἄνθρωπος ὁ θεὸς λέγει.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err(), "Expected error for double subject: {:?}", res);
    let err = res.unwrap_err();
    assert!(matches!(err, glossa::errors::GlossaError::AssemblyError(glossa::errors::AssemblyError::DoubleSubject)), "Expected DoubleSubject, got {:?}", err);
}"""

    old_test_2 = """#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let prog = analyze_program(&ast).unwrap();

    // The previous implementation was completely ignoring undefined variables and producing an empty print.
    // Let's assert it generates an empty print instead of crashing.
    if let glossa::semantic::AnalyzedStatement::Print(ref expressions) = prog.statements[0]
        && expressions.is_empty()
    {
        // It silently became empty/zero!
        return;
    }
    panic!(
        "Did not evaluate to empty/zero silently! It got {:?}",
        prog.statements[0]
    );
}"""

    new_test_2 = """#[test]
fn test_undefined_variable_evaluates_to_zero_silently() {
    let source = "ἄγνωστος λέγε."; // 'unknown say' -> undefined variable
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), glossa::errors::GlossaError::UndefinedName { .. }));
}"""

    old_test_3 = """#[test]
#[should_panic(expected = "MissingVerb")]
fn test_missing_verb_compiler_panic() {
    // Missing verb `ὁ ἄνθρωπος.` actually crashes `rustc` codegen if passed through,
    // or panics locally. We prove it panics or fails to compile!
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let _ = analyze_program(&ast).unwrap();
}"""

    new_test_3 = """#[test]
fn test_missing_verb_compiler_panic() {
    let source = "ὁ ἄνθρωπος.";
    let ast = parse(source).unwrap();
    let res = analyze_program(&ast);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), glossa::errors::GlossaError::AssemblyError(glossa::errors::AssemblyError::MissingVerb)));
}"""

    content = content.replace(old_test_1, new_test_1)
    content = content.replace(old_test_2, new_test_2)
    content = content.replace(old_test_3, new_test_3)
    with open("tests/havoc_issue_echo.rs", "w") as f:
        f.write(content)

    # 5. tests/havoc_repro.rs
    content = open("tests/havoc_repro.rs").read()
    old_code = """        let source = format!("
            λείτουργος ὁρίζειν · δός {} 0 ἄθροισμα.

            // Main
            λείτουργος λέγε.
        ", val);"""

    new_code = """        let source = format!("
            ἔστω ὁ λειτουργος.
            λειτουργος ὁρίζειν · δός {} 0 ἄθροισμα.

            // Main
            λειτουργος λέγε.
        ", val);"""

    content = content.replace(old_code, new_code)
    with open("tests/havoc_repro.rs", "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()
