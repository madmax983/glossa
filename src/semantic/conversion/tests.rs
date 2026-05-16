#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use crate::semantic::conversion::statements::*;
    use crate::semantic::{
        AnalyzedStatement, Constituent, Literal, assembly::AssembledStatement,
        model::AnalyzedExprKind, resolver::Scope, types::GlossaType,
    };

    #[test]
    fn test_classify_pop_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(), // not a pop verb
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_pop("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_pop_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ὠθεῖ".into(), // actually a push verb, let's use ἕλκεται for pop, but any string works for the missing subject test since it checks lemma first
                normalized: "ἕλκεται".into(),
                original: "ἕλκεται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        // The check inside classify_pop explicitly looks at the passed verb_lemma ("ἕλκεται" is pop)
        let result = classify_pop("ἕλκεται", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_push_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_push_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ὠθεῖ".into(),
                normalized: "ὠθεῖ".into(),
                original: "ὠθεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("ὠθεῖ", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_insert_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("λέγει", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_insert_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τίθησι".into(),
                normalized: "τίθησι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τίθησι", &asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_no_containment() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "δεῖ".into(),
                normalized: "δεῖ".into(),
                original: "δεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            has_containment_preposition: false,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_assertion_missing_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "δεῖ".into(),
                normalized: "δεῖ".into(),
                original: "δεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            has_containment_preposition: true,
            subject: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(),
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_missing_left_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: None, // Missing subject means left_expr will be None
            literals: vec![Literal::Number(5)],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_undefined_left_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "y".into(), // y is not defined in scope
                normalized: "y".into(),
                original: "y".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![Literal::Number(5)],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_equality_assertion_missing_right_expr() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἰσοῦται".into(),
                normalized: "ἰσοῦται".into(),
                original: "ἰσοῦται".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![], // Empty literals means right_expr will be None
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);
        let result = classify_equality_assertion(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_collection_mutation_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_collection_mutation(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_print_no_verb() {
        let asm_stmt = AssembledStatement {
            verb: None,
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_print_wrong_verb() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγει".into(), // not a print verb (λέγε is, but here testing the literal check)
                normalized: "λέγει".into(),
                original: "λέγει".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_print_binary_op_empty() {
        let asm_stmt = AssembledStatement {
            operators: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_binary_op(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_print_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_property_access(&asm_stmt, &mut scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_print_index_access_empty() {
        let asm_stmt = AssembledStatement {
            index_accesses: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_index_access(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_print_unwrap_empty() {
        let asm_stmt = AssembledStatement {
            unwraps: vec![],
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = try_print_unwrap(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_no_subject() {
        let asm_stmt = AssembledStatement {
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_no_genitives() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_owner_not_found() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![Constituent {
                lemma: "x".into(), // Not in scope
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_genitive_method_call_method_already_defined() {
        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "len".into(),
                normalized: "len".into(),
                original: "len".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            genitives: vec![Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::String);
        scope.define("len", GlossaType::Number); // Method name is already a defined variable in scope
        let result = try_parse_genitive_method_call(&asm_stmt, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_genitive_method_call_empty() {
        let asm_stmt = AssembledStatement {
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_genitive_method_call(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_unwrap_empty() {
        let asm_stmt = AssembledStatement {
            unwraps: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_unwrap(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_subject_empty() {
        let asm_stmt = AssembledStatement {
            subject: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_subject(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_genitive_method_empty() {
        let asm_stmt = AssembledStatement {
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_genitive_method(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_nominatives_empty() {
        let asm_stmt = AssembledStatement {
            nominatives: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_nominatives(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_property_access_empty() {
        let asm_stmt = AssembledStatement {
            property_accesses: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_property_access(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_index_access_empty() {
        let asm_stmt = AssembledStatement {
            index_accesses: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_index_access(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_array_empty() {
        let asm_stmt = AssembledStatement {
            arrays: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_array(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_binary_op_empty() {
        let asm_stmt = AssembledStatement {
            operators: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_binary_op(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_enum_from_object_empty() {
        let asm_stmt = AssembledStatement {
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_enum_from_object(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_literal_empty() {
        let asm_stmt = AssembledStatement {
            literals: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_literal(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_extract_object_fallback_empty() {
        let asm_stmt = AssembledStatement {
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = extract_object_fallback(&asm_stmt, &scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_property_access_print_owner_not_in_scope() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγε".into(),
                normalized: "λέγε".into(),
                original: "λέγε".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            genitives: vec![Constituent {
                lemma: "owner".into(),
                normalized: "owner".into(),
                original: "owner".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            subject: Some(Constituent {
                lemma: "prop".into(),
                normalized: "prop".into(),
                original: "prop".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_property_access_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_property_access_print_owner_not_struct() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "λέγε".into(),
                normalized: "λέγε".into(),
                original: "λέγε".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            genitives: vec![Constituent {
                lemma: "owner".into(),
                normalized: "owner".into(),
                original: "owner".into(),
                gender: None,
                case: crate::morphology::Case::Genitive,
                number: None,
                person: None,
            }],
            subject: Some(Constituent {
                lemma: "prop".into(),
                normalized: "prop".into(),
                original: "prop".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("owner", GlossaType::Number); // Not a struct
        let result = classify_property_access_print(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_function_call_no_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἔστω".into(), // Binding verb
                normalized: "ἔστω".into(),
                original: "ἔστω".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            object: Some(Constituent {
                lemma: "myfunc".into(),
                normalized: "myfunc".into(),
                original: "myfunc".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: None,
                person: None,
            }),
            subject: None, // No subject
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define_function("myfunc", vec![], Some(GlossaType::Number));
        let result = classify_function_call(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_classify_subjunctive_comparison_no_subject() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ἔστω".into(), // Binding verb
                normalized: "ἔστω".into(),
                original: "ἔστω".into(),
                person: None,
                number: None,
                tense: None,
                mood: Some(crate::morphology::Mood::Subjunctive),
                voice: None,
            }),
            operators: vec![crate::morphology::lexicon::BinaryOp::Eq],
            literals: vec![Literal::Number(5)],
            subject: None, // No subject
            ..Default::default()
        };
        let mut scope = Scope::new();
        let result = classify_subjunctive_comparison(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_resolve_binding_target_subject_object_swap() {
        // Create a scope where 'subject_var' IS defined but 'object_var' is NOT defined.
        // This should trigger the Subject/Object swap logic.
        let mut scope = Scope::new();
        scope.define("subject_var", GlossaType::Number);

        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "subject_var".into(),
                normalized: "subject_var".into(),
                original: "subject_var".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            object: Some(Constituent {
                lemma: "object_var".into(),
                normalized: "object_var".into(),
                original: "object_var".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            ..Default::default()
        };

        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        // Since 'subject_var' was defined and 'object_var' was not, it should bind to 'object_var'
        assert_eq!(name, "object_var");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));

        // Ensure they were actually swapped
        assert_eq!(fixed_asm.subject.as_ref().unwrap().lemma, "object_var");
        assert_eq!(fixed_asm.object.as_ref().unwrap().lemma, "subject_var");
    }

    #[test]
    fn test_resolve_binding_target_subject_object_no_swap() {
        // Create a scope where NEITHER is defined.
        // This should skip the swap logic and fall into the 'else' branch,
        // binding to the subject and returning Cow::Borrowed.
        let scope = Scope::new();

        let asm_stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "subject_var".into(),
                normalized: "subject_var".into(),
                original: "subject_var".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            object: Some(Constituent {
                lemma: "object_var".into(),
                normalized: "object_var".into(),
                original: "object_var".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: Some(crate::morphology::Number::Singular),
                person: None,
            }),
            ..Default::default()
        };

        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "subject_var");
        assert!(matches!(fixed_asm, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_resolve_binding_target_no_subject_has_participle() {
        // This tests the "Fallback: Bind to first participle (if any remain)" case
        // We use a verb_lemma that exists in the lexicon so it's NOT treated as a "false participle"
        // Let's use "λεγω" which is definitely a verb
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "λεγω".into(),
                normalized: "λεγων".into(), // Actual participle
                original: "λέγων".into(),
                gender: crate::morphology::Gender::Masculine,
                case: crate::morphology::Case::Nominative,
                number: crate::morphology::Number::Singular,
                voice: crate::morphology::Voice::Active,
                tense: crate::morphology::Tense::Present,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "λεγων");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));
        assert!(fixed_asm.participles.is_empty()); // Should have been consumed
    }

    #[test]
    fn test_resolve_binding_target_false_participle() {
        // This tests the "false participle" check at the very beginning of the function
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "not_a_real_verb_lemma".into(), // Will fail lexicon lookup
                normalized: "false_participle".into(),
                original: "false_participle".into(),
                gender: crate::morphology::Gender::Masculine,
                case: crate::morphology::Case::Nominative,
                number: crate::morphology::Number::Singular,
                voice: crate::morphology::Voice::Active,
                tense: crate::morphology::Tense::Present,
            }],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_ok());
        let (name, fixed_asm) = result.unwrap();
        assert_eq!(name, "false_participle");
        assert!(matches!(fixed_asm, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn test_resolve_binding_target_no_subject_no_participle() {
        let asm_stmt = AssembledStatement {
            subject: None,
            participles: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = resolve_binding_target(&asm_stmt, &scope);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Binding without subject")
        );
    }

    #[test]
    fn test_classify_query_containment_no_literal() {
        let asm_stmt = AssembledStatement {
            is_query: true,
            has_containment_preposition: true,
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![], // No literal element
            ..Default::default()
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::List(Box::new(GlossaType::Number)));
        let result = classify_query(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        let stmt = result.unwrap();
        assert!(stmt.is_some());

        // Ensure the fallback literal generation (0) happened
        if let AnalyzedStatement::Query(exprs) = stmt.unwrap() {
            assert_eq!(exprs.len(), 1);
            if let AnalyzedExprKind::MethodCall { args, .. } = &exprs[0].expr {
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::UnaryOp { op, operand } = &args[0].expr {
                    assert_eq!(*op, crate::morphology::lexicon::UnaryOp::Ref);
                    assert!(matches!(operand.expr, AnalyzedExprKind::NumberLiteral(0)));
                } else {
                    panic!("Expected UnaryOp Ref");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Query");
        }
    }

    #[test]
    fn test_classify_insert_no_args() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τιθημι".into(), // insert verb
                normalized: "τιθημι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![],
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τιθημι", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "insert");
                assert!(args.is_empty());
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_insert_object() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "τιθημι".into(), // insert verb
                normalized: "τιθημι".into(),
                original: "τίθησι".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            object: Some(Constituent {
                lemma: "y".into(),
                normalized: "y".into(),
                original: "y".into(),
                gender: None,
                case: crate::morphology::Case::Accusative,
                number: None,
                person: None,
            }),
            literals: vec![],
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_insert("τιθημι", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "insert");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::Variable(var_name) = &args[0].expr {
                    assert_eq!(var_name, "y");
                } else {
                    panic!("Expected Variable argument");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_push_no_args() {
        let asm_stmt = AssembledStatement {
            verb: Some(crate::semantic::assembly::VerbConstituent {
                lemma: "ωθω".into(), // push verb
                normalized: "ωθω".into(),
                original: "ὠθεῖ".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            subject: Some(Constituent {
                lemma: "x".into(),
                normalized: "x".into(),
                original: "x".into(),
                gender: None,
                case: crate::morphology::Case::Nominative,
                number: None,
                person: None,
            }),
            literals: vec![],
            object: None,
            ..Default::default()
        };
        let scope = Scope::new();
        let result = classify_push("ωθω", &asm_stmt, &scope);
        assert!(result.is_ok());
        let opt_stmt = result.unwrap();
        assert!(opt_stmt.is_some());
        if let AnalyzedStatement::Expression(exprs) = opt_stmt.unwrap() {
            if let AnalyzedExprKind::MethodCall { method, args, .. } = &exprs[0].expr {
                assert_eq!(method, "push");
                assert_eq!(args.len(), 1);
                if let AnalyzedExprKind::NumberLiteral(val) = args[0].expr {
                    assert_eq!(val, 0); // fallback is 0
                } else {
                    panic!("Expected NumberLiteral fallback argument");
                }
            } else {
                panic!("Expected MethodCall");
            }
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_classify_expression_empty_exprs_propagate() {
        let scope = Scope::new();
        // Create an AssembledStatement that will produce an empty `exprs` array
        // but has `is_propagate` set to true.
        let asm_stmt = AssembledStatement {
            is_propagate: true,
            ..Default::default()
        };
        // No literals, operators, subject, object, or nested phrases -> exprs will be empty.

        let result = classify_expression(&asm_stmt, &scope);
        assert!(result.is_ok());

        if let AnalyzedStatement::Expression(exprs) = result.unwrap() {
            assert!(exprs.is_empty(), "Expected empty expressions array");
        } else {
            panic!("Expected AnalyzedStatement::Expression");
        }
    }

    #[test]
    fn test_extract_unwrap_with_expr_for_coverage() {
        let scope = Scope::new();

        let asm_stmt = AssembledStatement {
            unwraps: vec![crate::ast::Expr::NumberLiteral(42)],
            ..Default::default()
        };
        let result = extract_unwrap(&asm_stmt, &scope);
        assert!(result.is_ok());
        let opt = result.unwrap();
        assert!(opt.is_some());

        let (expr, ty) = opt.unwrap();
        assert_eq!(ty, GlossaType::Unknown);
        if let AnalyzedExprKind::Unwrap(inner) = expr.expr {
            if let AnalyzedExprKind::NumberLiteral(n) = inner.expr {
                assert_eq!(n, 42);
            } else {
                panic!("Expected NumberLiteral inside Unwrap");
            }
        } else {
            panic!("Expected Unwrap expr");
        }
    }

    #[test]
    fn test_try_print_unwrap_with_expr_for_coverage() {
        let mut scope = Scope::new();

        let asm_stmt = AssembledStatement {
            unwraps: vec![crate::ast::Expr::NumberLiteral(42)],
            ..Default::default()
        };
        let result = try_print_unwrap(&asm_stmt, &mut scope);
        assert!(result.is_ok());
        let opt = result.unwrap();
        assert!(opt.is_some());

        let exprs = opt.unwrap();
        assert_eq!(exprs.len(), 1);

        if let AnalyzedExprKind::Unwrap(inner) = &exprs[0].expr {
            if let AnalyzedExprKind::NumberLiteral(n) = &inner.expr {
                assert_eq!(*n, 42);
            } else {
                panic!("Expected NumberLiteral inside Unwrap");
            }
        } else {
            panic!("Expected Unwrap expr");
        }
    }
}
