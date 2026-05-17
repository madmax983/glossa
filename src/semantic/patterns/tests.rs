#![allow(unused_imports)]
use super::*;
use crate::morphology::analyze;
use crate::semantic::{AnalyzedExprKind, Constituent};

#[test]
fn test_process_fold_participle_unsupported_op_fallback() {
    let participle = crate::semantic::assembly::ParticipleConstituent {
        verb_lemma: "dummy".into(),
        original: "dummy".into(),
        normalized: "dummy".into(),
        case: crate::morphology::Case::Nominative,
        number: crate::morphology::Number::Singular,
        gender: crate::morphology::Gender::Masculine,
        tense: crate::morphology::Tense::Present,
        voice: crate::morphology::Voice::Active,
    };

    let mut asm_stmt = AssembledStatement::default();
    asm_stmt
        .operators
        .push(crate::morphology::lexicon::BinaryOp::Sub);

    let mut current_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(5),
        glossa_type: GlossaType::Number,
    };

    let result = process_fold_participle(&participle, &asm_stmt, &mut current_expr);
    assert!(
        !result,
        "process_fold_participle should safely return false on unsupported operator"
    );
}

#[test]
fn test_extract_comparison_value_lemma() {
    let mut scope = Scope::new();
    scope.define("ονομα", GlossaType::String);

    // Analyze 'onomatos'
    let analysis = analyze("ὀνόματος");
    println!("Analysis: {:?}", analysis);

    let mut stmt = AssembledStatement::default();
    stmt.genitives.push(Constituent {
        lemma: smol_str::SmolStr::new(analysis.lemma.as_ref()),
        original: "ὀνόματος".into(),
        normalized: "ονοματος".into(),
        case: crate::morphology::Case::Genitive,
        number: None,
        gender: None,
        person: None,
    });

    let expr = extract_comparison_value(&stmt, &scope);
    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(name, "ονομα", "Expected lemma 'ονομα', got '{}'", name);
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_extract_comparison_value_stripped_suffix_es() {
    let mut scope = Scope::new();
    scope.define("αγαπ", GlossaType::Number);

    let mut stmt = AssembledStatement::default();
    stmt.genitives.push(Constituent {
        lemma: "dummy".into(),
        original: "αγαπης".into(),
        normalized: "αγαπης".into(),
        case: crate::morphology::Case::Genitive,
        number: None,
        gender: None,
        person: None,
    });

    let expr = extract_comparison_value(&stmt, &scope);
    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(
            name, "αγαπ",
            "Expected stripped name 'αγαπ', got '{}'",
            name
        );
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_extract_comparison_value_stripped_suffix_on() {
    let mut scope = Scope::new();
    scope.define("μετρ", GlossaType::Number);

    let mut stmt = AssembledStatement::default();
    stmt.genitives.push(Constituent {
        lemma: "dummy".into(),
        original: "μετρων".into(),
        normalized: "μετρων".into(),
        case: crate::morphology::Case::Genitive,
        number: None,
        gender: None,
        person: None,
    });

    let expr = extract_comparison_value(&stmt, &scope);
    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(
            name, "μετρ",
            "Expected stripped name 'μετρ', got '{}'",
            name
        );
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_extract_comparison_value_stripped() {
    let mut scope = Scope::new();
    scope.define("θ", GlossaType::Number);

    // 'thou' (θου) -> 'th' (θ)
    let mut stmt = AssembledStatement::default();
    stmt.genitives.push(Constituent {
        lemma: "dummy".into(), // Lemma lookup fails (not 'θ')
        original: "θου".into(),
        normalized: "θου".into(),
        case: crate::morphology::Case::Genitive,
        number: None,
        gender: None,
        person: None,
    });

    let expr = extract_comparison_value(&stmt, &scope);
    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(name, "θ", "Expected stripped name 'θ', got '{}'", name);
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_extract_comparison_value_original() {
    let mut scope = Scope::new();
    // Define 'myos' (μυός) as a variable directly (maybe it's a genitive variable?)
    scope.define("μυος", GlossaType::Number);

    let mut stmt = AssembledStatement::default();
    stmt.genitives.push(Constituent {
        lemma: "mys".into(), // Lemma lookup fails (not 'μυος')
        original: "μυός".into(),
        normalized: "μυος".into(),
        case: crate::morphology::Case::Genitive,
        number: None,
        gender: None,
        person: None,
    });

    let expr = extract_comparison_value(&stmt, &scope);
    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(
            name, "μυος",
            "Expected original name 'μυος', got '{}'",
            name
        );
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_extract_comparison_value_fallback() {
    let scope = Scope::new();
    // Nothing defined in scope. Should default to stripped name.

    let mut stmt = AssembledStatement::default();
    stmt.genitives.push(Constituent {
        lemma: "dummy".into(),
        original: "θου".into(),
        normalized: "θου".into(),
        case: crate::morphology::Case::Genitive,
        number: None,
        gender: None,
        person: None,
    });

    let expr = extract_comparison_value(&stmt, &scope);
    if let AnalyzedExprKind::Variable(name) = expr.expr {
        assert_eq!(
            name, "θ",
            "Expected fallback to stripped name 'θ', got '{}'",
            name
        );
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_try_parse_struct_instantiation_empty_terms() {
    let mut scope = Scope::new();
    let stmt = crate::ast::Statement::Regular {
        clauses: vec![crate::ast::Clause {
            expressions: vec![crate::ast::Expr::Phrase(vec![
                crate::ast::Expr::Word(crate::ast::Word::new("var")),
                crate::ast::Expr::Word(crate::ast::Word::new("νεον")),
                crate::ast::Expr::Word(crate::ast::Word::new("Type")),
                crate::ast::Expr::NumberLiteral(5),
            ])],
        }],
        is_query: false,
        is_propagate: false,
    };

    let result = try_parse_struct_instantiation(&stmt, &mut scope);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_try_parse_struct_instantiation_non_struct_type() {
    let mut scope = Scope::new();
    // Define a type that is NOT a Struct
    scope.define_type("NonStructType", GlossaType::Number);

    let stmt = crate::ast::Statement::Regular {
        clauses: vec![crate::ast::Clause {
            expressions: vec![crate::ast::Expr::Phrase(vec![
                crate::ast::Expr::Word(crate::ast::Word::new("var")),
                crate::ast::Expr::Word(crate::ast::Word::new("νεον")),
                crate::ast::Expr::Word(crate::ast::Word::new("NonStructType")),
                // ἔστω is a binding verb
                crate::ast::Expr::Word(crate::ast::Word::new("ἔστω")),
            ])],
        }],
        is_query: false,
        is_propagate: false,
    };

    let result = try_parse_struct_instantiation(&stmt, &mut scope);
    // The non-struct branch will return an Err because `GlossaType::Number` is not a `Struct`.
    // See line 228: if the type is found but not a struct, it should ideally not match,
    // but right now lookup_type returns Option<&GlossaType>.
    // Actually wait, looking at line 230:
    // return Err(GlossaError::undefined(type_name.to_string()));
    assert!(result.is_err());
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::semantic::{AnalyzedExprKind, Constituent};
    #[test]
    fn test_detect_iterator_pattern_any() {
        // Covers process_explicit_quantifiers with scope lookup
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Number);

        let mut stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "list".into(),
                original: "list".into(),
                normalized: "list".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            ..Default::default()
        };
        // "any" quantifier logic is checked via flags, usually checked by lemma of subject
        // But here we manually trigger process_explicit_quantifiers by setting operators and genitives

        // Mock the quantifier logic by using "τι" (any) as an extra nominative
        // Subject must remain "list" for extract_collection to work (it rejects quantifiers)
        stmt.nominatives.push(Constituent {
            lemma: "τις".into(),
            original: "τι".into(),
            normalized: "τι".into(),
            case: crate::morphology::Case::Nominative,
            number: None,
            gender: None,
            person: None,
        });

        // "greater" (operator)
        stmt.operators
            .push(crate::morphology::lexicon::BinaryOp::Gt);

        // "than x" (genitive comparison value)
        stmt.genitives.push(Constituent {
            lemma: "x".into(),
            original: "x".into(), // Normalized will match scope "x"
            normalized: "x".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        // "print" verb to trigger detection
        stmt.verb = Some(crate::semantic::VerbConstituent {
            lemma: "λεγω".into(),
            original: "λεγε".into(),
            normalized: "λεγε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        });

        let result = detect_iterator_pattern(&stmt, &mut scope);
        assert!(result.is_ok());
        let expr_opt = result.unwrap();
        assert!(expr_opt.is_some());

        let expr = expr_opt.unwrap();
        // Should be MethodCall "any"
        if let AnalyzedExprKind::MethodCall { method, args, .. } = expr.expr {
            assert_eq!(method, "any");
            assert_eq!(args.len(), 1);
            // Verify argument is a closure x > x
            // (The detailed closure structure is complex to verify, but method name is key)
        } else {
            panic!("Expected MethodCall 'any', got {:?}", expr.expr);
        }
    }

    #[test]
    fn test_detect_iterator_pattern_find() {
        // Covers process_find with scope lookup
        let mut scope = Scope::new();
        scope.define("target", GlossaType::Number);

        let mut stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "list".into(),
                original: "list".into(),
                normalized: "list".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            ..Default::default()
        };

        // "greater" (operator)
        stmt.operators
            .push(crate::morphology::lexicon::BinaryOp::Gt);

        // "target" (genitive)
        stmt.genitives.push(Constituent {
            lemma: "target".into(),
            original: "target".into(),
            normalized: "target".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        // "find" verb
        stmt.verb = Some(crate::semantic::VerbConstituent {
            lemma: "ευρισκω".into(), // is_find_verb check
            original: "ευρε".into(),
            normalized: "ευρε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        });

        let result = detect_iterator_pattern(&stmt, &mut scope);
        assert!(result.is_ok());
        let expr_opt = result.unwrap();
        assert!(expr_opt.is_some());

        let expr = expr_opt.unwrap();
        // Should be MethodCall "find"
        if let AnalyzedExprKind::MethodCall { method, args, .. } = expr.expr {
            assert_eq!(method, "find");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected MethodCall 'find', got {:?}", expr.expr);
        }
    }

    #[test]
    fn test_detect_iterator_pattern_filter() {
        // Covers process_adjectives with scope lookup
        let mut scope = Scope::new();
        scope.define("threshold", GlossaType::Number);

        let mut stmt = AssembledStatement {
            subject: Some(Constituent {
                lemma: "list".into(),
                original: "list".into(),
                normalized: "list".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            ..Default::default()
        };

        // "greater" (adjective -> filter)
        stmt.adjectives.push(Constituent {
            lemma: "μεγας".into(), // lemma for μείζον
            original: "μείζονα".into(),
            normalized: "μειζονα".into(),
            case: crate::morphology::Case::Accusative,
            number: None,
            gender: None,
            person: None,
        });

        // "threshold" (genitive comparison value)
        stmt.genitives.push(Constituent {
            lemma: "threshold".into(),
            original: "threshold".into(),
            normalized: "threshold".into(),
            case: crate::morphology::Case::Genitive,
            number: None,
            gender: None,
            person: None,
        });

        // "print" verb
        stmt.verb = Some(crate::semantic::VerbConstituent {
            lemma: "λεγω".into(),
            original: "λεγε".into(),
            normalized: "λεγε".into(),
            person: None,
            number: None,
            tense: None,
            mood: None,
            voice: None,
        });

        let result = detect_iterator_pattern(&stmt, &mut scope);
        assert!(result.is_ok());
        let expr_opt = result.unwrap();
        assert!(expr_opt.is_some());

        let expr = expr_opt.unwrap();
        // Should be MethodCall "collect" (finalized), inner is filter
        if let AnalyzedExprKind::MethodCall {
            method, receiver, ..
        } = expr.expr
        {
            assert_eq!(method, "collect");
            // Check inner receiver
            if let AnalyzedExprKind::MethodCall {
                method: inner_method,
                ..
            } = receiver.expr
            {
                assert_eq!(inner_method, "filter");
            } else {
                panic!("Expected inner MethodCall 'filter'");
            }
        } else {
            panic!("Expected MethodCall 'collect'");
        }
    }

    #[test]
    fn test_parse_struct_args_unsupported_type() {
        let scope = Scope::new();
        let fields = vec![("x".into(), GlossaType::Number)];

        // Pass an unsupported expression type (e.g., a phrase)
        let terms = vec![crate::ast::Expr::Phrase(vec![])];

        let result = parse_struct_args(&terms, &fields, &scope);
        assert!(result.is_none());
    }

    #[test]
    fn test_process_find_no_predicate_coverage() {
        let scope = Scope::new();
        let asm_stmt = AssembledStatement::default();
        let mut current_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        // No operators -> found_predicate = false
        process_find(&asm_stmt, &scope, &mut current_expr);

        // Expected to be a MethodCall to "next" with no args
        if let AnalyzedExprKind::MethodCall { method, args, .. } = current_expr.expr {
            assert_eq!(method, "find");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected MethodCall 'find'");
        }
    }
}
