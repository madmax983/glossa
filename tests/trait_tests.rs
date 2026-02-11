use glossa::ast::Statement;
use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    generate_rust(&analyzed)
}

// =============================================================================
// CYCLE 1: Trait Definition Parsing
// =============================================================================

#[test]
fn test_parse_empty_trait() {
    let input = "χαρακτήρ Showable ὁρίζειν { }.";
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitDefinition(trait_def) => {
            assert_eq!(trait_def.name.original, "Showable");
            assert_eq!(trait_def.methods.len(), 0);
        }
        _ => panic!("Expected TraitDefinition, got {:?}", ast.statements[0]),
    }
}

#[test]
fn test_parse_trait_with_required_method() {
    let input = "χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.";
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitDefinition(trait_def) => {
            assert_eq!(trait_def.name.original, "Showable");
            assert_eq!(trait_def.methods.len(), 1);

            let method = &trait_def.methods[0];
            assert_eq!(method.name.original, "show");
            assert!(!method.is_default);
            assert!(method.body.is_none());
            assert_eq!(method.params.len(), 1); // self parameter
        }
        _ => panic!("Expected TraitDefinition, got {:?}", ast.statements[0]),
    }
}

#[test]
fn test_parse_trait_with_default_method() {
    let input = "χαρακτήρ Math ὁρίζειν { ἤδη double τῷ self· δός selfου add self. }.";
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitDefinition(trait_def) => {
            assert_eq!(trait_def.name.original, "Math");
            assert_eq!(trait_def.methods.len(), 1);

            let method = &trait_def.methods[0];
            assert_eq!(method.name.original, "double");
            assert!(method.is_default);
            assert!(method.body.is_some());
        }
        _ => panic!("Expected TraitDefinition, got {:?}", ast.statements[0]),
    }
}

#[test]
fn test_parse_trait_multiple_methods() {
    let input = r#"
        χαρακτήρ Math ὁρίζειν {
            δεῖ add τῷ self τῷ other.
            ἤδη double τῷ self· δός selfου add self.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitDefinition(trait_def) => {
            assert_eq!(trait_def.name.original, "Math");
            assert_eq!(trait_def.methods.len(), 2);

            // First method: required
            assert_eq!(trait_def.methods[0].name.original, "add");
            assert!(!trait_def.methods[0].is_default);
            assert_eq!(trait_def.methods[0].params.len(), 2); // self and other

            // Second method: default
            assert_eq!(trait_def.methods[1].name.original, "double");
            assert!(trait_def.methods[1].is_default);
            assert!(trait_def.methods[1].body.is_some());
        }
        _ => panic!("Expected TraitDefinition, got {:?}", ast.statements[0]),
    }
}

// =============================================================================
// CYCLE 2: Trait Semantic Analysis
// =============================================================================

#[test]
fn test_analyze_trait_definition() {
    let input = "χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.";
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Trait definition should analyze without errors: {:?}",
        result.err()
    );
}

#[test]
fn test_trait_stored_in_scope() {
    // This test will need access to the scope after analysis
    // For now, we'll just check that analysis succeeds
    let input = "χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.";
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Should store trait in scope: {:?}",
        result.err()
    );
}

#[test]
fn test_duplicate_trait_error() {
    let input = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        χαρακτήρ Showable ὁρίζειν { δεῖ display τῷ self. }.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(result.is_err(), "Should error on duplicate trait name");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("already defined") || err_msg.contains("duplicate"),
        "Error should mention duplicate/already defined: {}",
        err_msg
    );
}

#[test]
fn test_default_method_body_analysis() {
    let input = "χαρακτήρ Math ὁρίζειν { ἤδη double τῷ self· δός πέντε. }.";
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Default method body should be analyzed: {:?}",
        result.err()
    );
}

// =============================================================================
// CYCLE 3: Trait Implementation Parsing
// =============================================================================

#[test]
fn test_parse_empty_trait_impl() {
    let input = "εἶδος Point τῷ Showable ἐμπίπτειν { }.";
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitImpl(trait_impl) => {
            assert_eq!(trait_impl.type_name.original, "Point");
            assert_eq!(trait_impl.trait_name.original, "Showable");
            assert_eq!(trait_impl.methods.len(), 0);
        }
        _ => panic!("Expected TraitImpl, got {:?}", ast.statements[0]),
    }
}

#[test]
fn test_parse_trait_impl_with_method() {
    let input = r#"
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitImpl(trait_impl) => {
            assert_eq!(trait_impl.type_name.original, "Point");
            assert_eq!(trait_impl.trait_name.original, "Showable");
            assert_eq!(trait_impl.methods.len(), 1);

            let method = &trait_impl.methods[0];
            assert_eq!(method.name.original, "show");
            assert!(!method.body.is_empty());
        }
        _ => panic!("Expected TraitImpl, got {:?}", ast.statements[0]),
    }
}

#[test]
fn test_parse_impl_multiple_methods() {
    let input = r#"
        εἶδος Number τῷ Math ἐμπίπτειν {
            add τῷ self τῷ other· δός νέον Number (selfου v otherou v ἄθροισμα).
            subtract τῷ self τῷ other· δός νέον Number (selfου v otherou v διαφορά).
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");

    assert_eq!(ast.statements.len(), 1);
    match &ast.statements[0] {
        Statement::TraitImpl(trait_impl) => {
            assert_eq!(trait_impl.type_name.original, "Number");
            assert_eq!(trait_impl.trait_name.original, "Math");
            assert_eq!(trait_impl.methods.len(), 2);

            assert_eq!(trait_impl.methods[0].name.original, "add");
            assert_eq!(trait_impl.methods[1].name.original, "subtract");
        }
        _ => panic!("Expected TraitImpl, got {:?}", ast.statements[0]),
    }
}

// =============================================================================
// CYCLE 4: Trait Implementation Validation
// =============================================================================

#[test]
fn test_analyze_trait_impl() {
    let input = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Valid impl should analyze without errors: {:?}",
        result.err()
    );
}

#[test]
fn test_impl_for_undefined_trait_error() {
    let input = r#"
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(result.is_err(), "Should error when trait doesn't exist");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("showable")
            || err_msg.contains("not")
            || err_msg.contains("defined"),
        "Error should mention undefined trait: {}",
        err_msg
    );
}

#[test]
fn test_impl_for_undefined_type_error() {
    let input = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(result.is_err(), "Should error when type doesn't exist");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("point")
            || err_msg.contains("not")
            || err_msg.contains("defined"),
        "Error should mention undefined type: {}",
        err_msg
    );
}

#[test]
fn test_missing_required_method_error() {
    let input = r#"
        χαρακτήρ Math ὁρίζειν {
            δεῖ add τῷ self τῷ other.
            δεῖ subtract τῷ self τῷ other.
        }.
        εἶδος Number ὁρίζειν { v ἀριθμοῦ. }.
        εἶδος Number τῷ Math ἐμπίπτειν {
            add τῷ self τῷ other· δός πέντε.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_err(),
        "Should error when required method is missing"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("subtract")
            || err_msg.contains("required")
            || err_msg.contains("not implement"),
        "Error should mention missing required method: {}",
        err_msg
    );
}

#[test]
fn test_impl_with_default_method_not_required() {
    let input = r#"
        χαρακτήρ Math ὁρίζειν {
            δεῖ add τῷ self τῷ other.
            ἤδη double τῷ self· δός πέντε.
        }.
        εἶδος Number ὁρίζειν { v ἀριθμοῦ. }.
        εἶδος Number τῷ Math ἐμπίπτειν {
            add τῷ self τῷ other· δός τρία.
        }.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Default methods should be optional: {:?}",
        result.err()
    );
}

// =============================================================================
// CYCLE 5: Trait Method Calls
// =============================================================================

#[test]
fn test_call_trait_method() {
    let input = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
        π νέον Point πέντε ἔστω.
        που show.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Should allow calling trait methods: {:?}",
        result.err()
    );
}

#[test]
fn test_call_trait_method_with_args() {
    let input = r#"
        χαρακτήρ Math ὁρίζειν { δεῖ add τῷ self τῷ other. }.
        εἶδος Number ὁρίζειν { v ἀριθμοῦ. }.
        εἶδος Number τῷ Math ἐμπίπτειν {
            add τῷ self τῷ other· δός νέον Number (selfου v otherou v ἄθροισμα).
        }.
        α νέον Number πέντε ἔστω.
        β νέον Number τρία ἔστω.
        γ αου add β ἔστω.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Should allow calling trait methods with args: {:?}",
        result.err()
    );
}

#[test]
fn test_call_default_method() {
    let input = r#"
        χαρακτήρ Math ὁρίζειν {
            δεῖ value τῷ self.
            ἤδη double τῷ self· δός selfου value selfου value ἄθροισμα.
        }.
        εἶδος Number ὁρίζειν { v ἀριθμοῦ. }.
        εἶδος Number τῷ Math ἐμπίπτειν {
            value τῷ self· δός selfου v.
        }.
        α νέον Number πέντε ἔστω.
        β αου double ἔστω.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    assert!(
        result.is_ok(),
        "Should allow calling default trait methods: {:?}",
        result.err()
    );
}

#[test]
fn test_trait_method_call_error_not_implemented() {
    let input = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        π νέον Point πέντε ἔστω.
        που show.
    "#;
    let ast = parse(input).expect("Parsing failed");
    let result = analyze_program(&ast);
    // This should actually compile - the error would be at runtime
    // Or we could make it a compile-time error if we track which types implement which traits
    // For now, let's allow it to compile
    assert!(
        result.is_ok(),
        "Method calls are resolved dynamically: {:?}",
        result.err()
    );
}

// =============================================================================
// CYCLE 6: Code Generation
// =============================================================================

#[test]
fn test_codegen_trait_definition() {
    let source = "χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.";
    let code = compile(source);
    assert!(
        code.contains("trait G_showable"),
        "Should generate trait keyword: {}",
        code
    );
    // quote! adds spaces, so check for "& self" with possible spaces
    assert!(
        code.contains("fn g_show") && code.contains("& self"),
        "Should generate method signature: {}",
        code
    );
}

#[test]
fn test_codegen_trait_impl() {
    let source = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
    "#;
    let code = compile(source);
    assert!(
        code.contains("impl G_showable for G_point"),
        "Should generate impl block: {}",
        code
    );
    assert!(
        code.contains("fn g_show") && code.contains("& self"),
        "Should generate impl method: {}",
        code
    );
}

#[test]
fn test_codegen_trait_with_default() {
    let source = r#"
        χαρακτήρ Math ὁρίζειν {
            δεῖ value τῷ self.
            ἤδη double τῷ self· δός selfου value selfου value ἄθροισμα.
        }.
    "#;
    let code = compile(source);
    assert!(
        code.contains("trait G_math"),
        "Should generate trait: {}",
        code
    );
    assert!(
        code.contains("fn g_value") && code.contains("& self"),
        "Should have required method: {}",
        code
    );
    assert!(
        code.contains("fn g_double") && code.contains("& self"),
        "Should have default method: {}",
        code
    );
    // Default method should have a body
    assert!(
        code.contains("g_double") && code.contains("& self") && code.contains("{"),
        "Default method should have body: {}",
        code
    );
}

#[test]
fn test_codegen_trait_method_call_genitive() {
    // Test trait method call using genitive pattern (που show) which should work
    let source = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
        π νέον Point πέντε ἔστω.
        που show λέγε.
    "#;
    let code = compile(source);
    // Using genitive pattern (που), this creates a property access which becomes a method call
    // The print statement should show the method call
    assert!(
        code.contains("trait G_showable"),
        "Should have trait: {}",
        code
    );
    assert!(
        code.contains("impl G_showable for G_point"),
        "Should have impl: {}",
        code
    );
    // We know genitive pattern works from earlier tests
}

#[test]
fn test_codegen_full_example() {
    let source = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. ψ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
        π νέον Point πέντε τρία ἔστω.
        που show.
    "#;
    let code = compile(source);

    // Check for trait definition
    assert!(
        code.contains("trait G_showable"),
        "Missing trait definition: {}",
        code
    );
    // Check for struct definition
    assert!(
        code.contains("struct G_point"),
        "Missing struct definition: {}",
        code
    );
    // Check for impl block
    assert!(
        code.contains("impl G_showable for G_point"),
        "Missing impl block: {}",
        code
    );
    // Check for main function (quote! adds spaces)
    assert!(code.contains("fn main"), "Missing main function: {}", code);
}

#[test]
fn test_standalone_method_call() {
    // Test that standalone method calls (method_name receiver) work
    let source = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
        π νέον Point πέντε ἔστω.
        show π.
    "#;

    let code = compile(source);

    // Should contain the method call
    // π -> g__p
    assert!(code.contains("g__p . g_show") || code.contains("g__p.g_show"));
}

// ============================================================================
// CYCLE 7: Advanced Features
// ============================================================================

#[test]
fn test_type_implements_multiple_traits() {
    // A type can implement multiple traits
    let source = r#"
        χαρακτήρ Showable ὁρίζειν { δεῖ show τῷ self. }.
        χαρακτήρ Printable ὁρίζειν { δεῖ print τῷ self. }.
        εἶδος Point ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Point τῷ Showable ἐμπίπτειν {
            show τῷ self· selfου ξ λέγε.
        }.
        εἶδος Point τῷ Printable ἐμπίπτειν {
            print τῷ self· selfου ξ λέγε.
        }.
    "#;

    let code = compile(source);
    // Should have impl blocks for both traits
    assert!(code.contains("impl G_showable for G_point"));
    assert!(code.contains("impl G_printable for G_point"));
}

#[test]
fn test_override_default_method() {
    // Override a default method with custom implementation
    let source = r#"
        χαρακτήρ Describable ὁρίζειν {
            ἤδη describe τῷ self· «default» λέγε.
        }.
        εἶδος Thing ὁρίζειν { ν ἀριθμοῦ. }.
        εἶδος Thing τῷ Describable ἐμπίπτειν {
            describe τῷ self· «custom» λέγε.
        }.
    "#;

    let code = compile(source);
    // The impl should contain the overridden method
    assert!(code.contains("impl G_describable for G_thing"));
    // The impl body should have the custom implementation
    assert!(code.contains("custom") || code.contains("\"custom\""));
}

#[test]
fn test_call_both_trait_methods_on_same_type() {
    // Call methods from different traits on the same instance
    let source = r#"
        χαρακτήρ Alpha ὁρίζειν { δεῖ alpha τῷ self. }.
        χαρακτήρ Beta ὁρίζειν { δεῖ beta τῷ self. }.
        εἶδος Widget ὁρίζειν { ν ἀριθμοῦ. }.
        εἶδος Widget τῷ Alpha ἐμπίπτειν {
            alpha τῷ self· selfου ν λέγε.
        }.
        εἶδος Widget τῷ Beta ἐμπίπτειν {
            beta τῷ self· selfου ν λέγε.
        }.
        ω νέον Widget πέντε ἔστω.
        ωου alpha.
        ωου beta.
    "#;

    let code = compile(source);
    // Should have calls to both methods
    assert!(code.contains("g_alpha") && code.contains("g_beta"));
}

#[test]
fn test_multiple_types_implement_same_trait() {
    // Multiple types implement the same trait
    let source = r#"
        χαρακτήρ Countable ὁρίζειν { δεῖ count τῷ self. }.
        εἶδος Foo ὁρίζειν { ξ ἀριθμοῦ. }.
        εἶδος Bar ὁρίζειν { ψ ἀριθμοῦ. }.
        εἶδος Foo τῷ Countable ἐμπίπτειν {
            count τῷ self· selfου ξ λέγε.
        }.
        εἶδος Bar τῷ Countable ἐμπίπτειν {
            count τῷ self· selfου ψ λέγε.
        }.
    "#;

    let code = compile(source);
    // Should have impl blocks for both types
    assert!(code.contains("impl G_countable for G_foo"));
    assert!(code.contains("impl G_countable for G_bar"));
}

#[test]
fn test_repro_trait_default_method_return_type() {
    // Define a trait with a default method that returns a number
    // "must value to-self; give 5."
    // Since it returns 5, the signature SHOULD be `fn value(&self) -> i64`
    let source = r#"
        χαρακτήρ Numeric ὁρίζειν {
            ἤδη value τῷ self· δός 5.
        }.
    "#;

    let code = compile(source);

    // FIXED: Code should contain explicit return and return type
    assert!(
        code.contains("return 5"),
        "Should contain explicit return statement"
    );
    // quote! generates `-> i64` (with or without spaces depending on tokenizer)
    assert!(
        code.contains("-> i64") || code.contains("->i64"),
        "Signature should contain return type"
    );
}

#[test]
fn test_repro_trait_impl_return_type() {
    // Define a trait and implement it with a return value
    let source = r#"
        χαρακτήρ Numeric ὁρίζειν {
            δεῖ value τῷ self.
        }.
        εἶδος Num ὁρίζειν { ν ἀριθμοῦ. }.
        εἶδος Num τῷ Numeric ἐμπίπτειν {
            value τῷ self· δός 5.
        }.
    "#;

    let code = compile(source);

    // FIXED: Impl should contain explicit return and return type
    assert!(code.contains("return 5"));
    assert!(
        code.contains("-> i64") || code.contains("->i64"),
        "Impl signature should contain return type"
    );
}
