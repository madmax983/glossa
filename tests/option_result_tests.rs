#![allow(missing_docs)]
/// Integration tests for Option<T> and Result<T,E> types in GLOSSA
///
/// Tests the Ancient Greek morphology mapping:
/// - Optative mood → Option<T>
/// - οὐδέν (ouden) → None
/// - τί (ti) → Some
/// - ἐπιτυχία (epitychia) → Ok
/// - σφάλμα (sphalma) → Err
use glossa::codegen::to_rust_type;
use glossa::semantic::GlossaType;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> Result<String, String> {
    use glossa::codegen::generate_rust;
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    let ast = parse(source).map_err(|e| e.to_string())?;
    let analyzed = analyze_program(&ast).map_err(|e| e.to_string())?;
    Ok(generate_rust(&analyzed))
}

// ============================================================================
// Phase 1: Type System Foundation Tests
// ============================================================================

#[test]
fn test_option_type_exists() {
    let opt = GlossaType::Option(Box::new(GlossaType::Number));
    assert!(matches!(opt, GlossaType::Option(_)));
}

#[test]
fn test_result_type_exists() {
    let result = GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String));
    assert!(matches!(result, GlossaType::Result(_, _)));
}

#[test]
fn test_option_to_rust() {
    let opt = GlossaType::Option(Box::new(GlossaType::Number));
    assert_eq!(to_rust_type(&opt), "Option<i64>");
}

#[test]
fn test_result_to_rust() {
    let result = GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String));
    assert_eq!(to_rust_type(&result), "Result<i64, String>");
}

#[test]
fn test_option_type_compatibility() {
    let opt_num = GlossaType::Option(Box::new(GlossaType::Number));
    let opt_unknown = GlossaType::Option(Box::new(GlossaType::Unknown));
    assert!(opt_num.is_compatible(&opt_unknown));
}

#[test]
fn test_result_type_compatibility() {
    let res1 = GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String));
    let res2 = GlossaType::Result(Box::new(GlossaType::Unknown), Box::new(GlossaType::Unknown));
    assert!(res1.is_compatible(&res2));
}

#[test]
fn test_nested_option() {
    let nested = GlossaType::Option(Box::new(GlossaType::Option(Box::new(GlossaType::Number))));
    assert_eq!(to_rust_type(&nested), "Option<Option<i64>>");
}

// ============================================================================
// Phase 2: Optative Mood Analysis Tests
// ============================================================================

#[test]
fn test_optative_present_active() {
    use glossa::morphology::{Mood, Person, analyze};

    // γράφοιμι - "I might write" (present active optative)
    let analysis = analyze("γραφοιμι");
    assert_eq!(analysis.mood, Some(Mood::Optative));
    assert_eq!(analysis.person, Some(Person::First));
}

#[test]
fn test_optative_aorist_passive() {
    use glossa::morphology::{Mood, Person, analyze};

    // εὑρεθείη - "might be found" (aorist passive optative)
    let analysis = analyze("ευρεθειη");
    assert_eq!(analysis.mood, Some(Mood::Optative));
    assert_eq!(analysis.person, Some(Person::Third));
}

// ============================================================================
// Phase 3: Lexicon Vocabulary Tests
// ============================================================================

#[test]
fn test_ouden_is_none() {
    let entry = glossa::morphology::lookup("ουδεν");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().rust_equiv, Some("None"));
}

#[test]
fn test_ti_is_some() {
    let entry = glossa::morphology::lookup("τι");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().rust_equiv, Some("Some"));
}

#[test]
fn test_epitychia_is_ok() {
    let entry = glossa::morphology::lookup("επιτυχια");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().rust_equiv, Some("Ok"));
}

#[test]
fn test_sphalma_is_err() {
    let entry = glossa::morphology::lookup("σφαλμα");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().rust_equiv, Some("Err"));
}

#[test]
fn test_is_none_word_helper() {
    use glossa::morphology::is_none_word;

    assert!(is_none_word("ουδεν"));
    assert!(!is_none_word("τι"));
}

#[test]
fn test_is_some_word_helper() {
    use glossa::morphology::is_some_word;

    assert!(is_some_word("τι"));
    assert!(!is_some_word("ουδεν"));
}

#[test]
fn test_is_ok_word_helper() {
    use glossa::morphology::is_ok_word;

    assert!(is_ok_word("επιτυχια"));
    assert!(!is_ok_word("σφαλμα"));
}

#[test]
fn test_is_err_word_helper() {
    use glossa::morphology::is_err_word;

    assert!(is_err_word("σφαλμα"));
    assert!(!is_err_word("επιτυχια"));
}

// ============================================================================
// Phase 4: HIR Extensions Tests
// ============================================================================

// HIR tests removed as HIR layer was removed.

// ============================================================================
// Phase 5: Semantic Analysis Tests
// ============================================================================

#[test]
fn test_none_expression_analyzed() {
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    // ξ οὐδέν ἔστω. → let x = None;
    let source = "ξ ουδεν εστω.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    // Should have one binding statement
    assert_eq!(analyzed.statements.len(), 1);
}

#[test]
fn test_some_expression_analyzed() {
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    // ξ τί πέντε ἔστω. → let x = Some(5);
    let source = "ξ τι πεντε εστω.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    // Should have one binding statement
    assert_eq!(analyzed.statements.len(), 1);
}

#[test]
fn test_ok_expression_analyzed() {
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    // ξ ἐπιτυχία πέντε ἔστω. → let x = Ok(5);
    let source = "ξ επιτυχια πεντε εστω.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    // Should have one binding statement
    assert_eq!(analyzed.statements.len(), 1);
}

#[test]
fn test_err_expression_analyzed() {
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    // ξ σφάλμα «πρόβλημα» ἔστω. → let x = Err("problem");
    let source = "ξ σφαλμα «προβλημα» εστω.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();

    // Should have one binding statement
    assert_eq!(analyzed.statements.len(), 1);
}

#[test]
fn test_none_codegen() {
    // ξ οὐδέν ἔστω. → let x = None;
    let source = "ξ ουδεν εστω.";
    let output = compile(source).unwrap();

    // Should contain None
    assert!(
        output.contains("None"),
        "Expected 'None' in output: {}",
        output
    );
}

#[test]
fn test_some_codegen() {
    //ξ τί πέντε ἔστω. → let x = Some(5);
    let source = "ξ τι πεντε εστω.";
    let output = compile(source).unwrap();

    // Should contain Some(5)
    assert!(
        output.contains("Some"),
        "Expected 'Some' in output: {}",
        output
    );
    assert!(output.contains("5"), "Expected '5' in output: {}", output);
}

#[test]
fn test_ok_codegen() {
    //ξ ἐπιτυχία πέντε ἔστω. → let x = Ok(5);
    let source = "ξ επιτυχια πεντε εστω.";
    let output = compile(source).unwrap();

    // Should contain Ok(5)
    assert!(output.contains("Ok"), "Expected 'Ok' in output: {}", output);
    assert!(output.contains("5"), "Expected '5' in output: {}", output);
}

#[test]
fn test_err_codegen() {
    //ξ σφάλμα «πρόβλημα» ἔστω. → let x = Err("problem");
    let source = "ξ σφαλμα «προβλημα» εστω.";
    let output = compile(source).unwrap();

    // Should contain Err
    assert!(
        output.contains("Err"),
        "Expected 'Err' in output: {}",
        output
    );
}

// ============================================================================
// Phase 6: Propagation & Unwrap Operators Tests
// ============================================================================

#[test]
fn test_unwrap_operator_codegen() {
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    // ξ τί πέντε ἔστω. ξ! λέγε. → let x = Some(5); println!("{}", x.unwrap());
    let source = "ξ τι πεντε εστω. ξ! λεγε.";
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast);

    // Should analyze without errors
    assert!(analyzed.is_ok(), "Analysis failed: {:?}", analyzed.err());
}

#[test]
fn test_unwrap_in_expression() {
    // Test that ! suffix generates .unwrap()
    let source = "ξ τι πεντε εστω. ψ ξ! εστω.";
    let output = compile(source).unwrap();

    // Should contain unwrap
    assert!(
        output.contains("expect"),
        "Expected 'expect' in output: {}",
        output
    );
}

// ============================================================================
// Phase 7: Result Type - Comprehensive Tests
// ============================================================================

#[test]
fn test_ok_expression_exact_output() {
    // ξ ἐπιτυχία πέντε ἔστω. → let x = Ok(5);
    let source = "ξ επιτυχια πεντε εστω.";
    let output = compile(source).unwrap();

    // Check for Ok with 5 (allowing for spaces from quote! macro)
    assert!(
        output.contains("Ok") && output.contains("5i64"),
        "Expected 'Ok' with '5i64' in output: {}",
        output
    );
}

#[test]
fn test_err_expression_exact_output() {
    // ξ σφάλμα «πρόβλημα» ἔστω. → let x = Err("problem");
    let source = "ξ σφαλμα «προβλημα» εστω.";
    let output = compile(source).unwrap();

    assert!(
        output.contains("Err") && output.contains("προβλημα"),
        "Expected 'Err(\"προβλημα\")' in output: {}",
        output
    );
}

#[test]
fn test_result_with_number_value() {
    // Multiple Result values
    let source = r#"
        ξ επιτυχια δεκα εστω.
        ψ σφαλμα «λαθος» εστω.
    "#;
    let output = compile(source).unwrap();

    assert!(
        output.contains("Ok") && output.contains("10"),
        "Expected 'Ok' with '10' in output: {}",
        output
    );
    assert!(
        output.contains("Err"),
        "Expected 'Err' in output: {}",
        output
    );
}

#[test]
fn test_result_unwrap() {
    // ξ ἐπιτυχία πέντε ἔστω. ξ! λέγε. → let x = Ok(5); println!("{}", x.unwrap());
    let source = "ξ επιτυχια πεντε εστω. ξ! λεγε.";
    let output = compile(source).unwrap();

    assert!(
        output.contains("Ok") && output.contains("5"),
        "Expected 'Ok' with '5' in output: {}",
        output
    );
    assert!(
        output.contains("expect"),
        "Expected 'expect' in output: {}",
        output
    );
}

#[test]
fn test_result_print_directly() {
    // Print Result directly (not unwrapped)
    let source = "ξ επιτυχια πεντε εστω. ξ λεγε.";
    let output = compile(source).unwrap();

    assert!(
        output.contains("Ok") && output.contains("5"),
        "Expected 'Ok' with '5' in output: {}",
        output
    );
    // Should print the Result value itself, not unwrapped (allowing for space in "println !")
    assert!(
        output.contains("println"),
        "Expected println in output: {}",
        output
    );
}

#[test]
fn test_err_with_string_literal() {
    // Err with explicit string message
    let source = "αποτελεσμα σφαλμα «Αποτυχια» εστω.";
    let output = compile(source).unwrap();

    assert!(
        output.contains("Err") && output.contains("Αποτυχια"),
        "Expected 'Err(\"Αποτυχια\")' in output: {}",
        output
    );
}

#[test]
fn test_multiple_results_in_sequence() {
    // Multiple Result bindings
    let source = r#"
        α επιτυχια πεντε εστω.
        β σφαλμα «κακος» εστω.
        γ επιτυχια δεκα εστω.
    "#;
    let output = compile(source).unwrap();

    // Should have multiple Result expressions (checking for "Ok " with space)
    let ok_count = output.matches("Ok ").count();
    let err_count = output.matches("Err ").count();

    assert!(
        ok_count >= 2,
        "Expected at least 2 Ok calls, got {}: {}",
        ok_count,
        output
    );
    assert!(
        err_count >= 1,
        "Expected at least 1 Err call, got {}: {}",
        err_count,
        output
    );
}

#[test]
fn test_result_parallels_option() {
    // Verify Result behaves like Option
    let option_source = "ξ τι πεντε εστω.";
    let result_source = "ξ επιτυχια πεντε εστω.";

    let option_output = compile(option_source).unwrap();
    let result_output = compile(result_source).unwrap();

    // Both should compile successfully (allowing for spaces in output)
    assert!(
        option_output.contains("Some") && option_output.contains("5"),
        "Expected 'Some' with '5' in: {}",
        option_output
    );
    assert!(
        result_output.contains("Ok") && result_output.contains("5"),
        "Expected 'Ok' with '5' in: {}",
        result_output
    );
}

// ============================================================================
// Phase 8: Code Generation Refinements
// ============================================================================

// NOTE: Nested Option/Result constructions (e.g., Ok(Some(5))) are not yet supported
// The current implementation treats each constructor independently
// These are documented as future enhancements

#[test]
fn test_option_none_generates_correctly() {
    // None is standalone, not wrapped in Ok/Err
    let source = "ξ ουδεν εστω.";
    let output = compile(source).unwrap();

    assert!(output.contains("None"), "Expected 'None' in: {}", output);
    // Should NOT be wrapped in Ok or Err
    assert!(
        !output.contains("Ok"),
        "Should not contain 'Ok': {}",
        output
    );
    assert!(
        !output.contains("Err"),
        "Should not contain 'Err': {}",
        output
    );
}

#[test]
fn test_result_ok_generates_correctly() {
    // Ok wraps the value
    let source = "ξ επιτυχια πεντε εστω.";
    let output = compile(source).unwrap();

    assert!(output.contains("Ok"), "Expected 'Ok' in: {}", output);
    assert!(output.contains("5"), "Expected '5' in: {}", output);
}

#[test]
fn test_result_err_generates_correctly() {
    // Err wraps the value
    let source = "ξ σφαλμα πεντε εστω.";
    let output = compile(source).unwrap();

    assert!(output.contains("Err"), "Expected 'Err' in: {}", output);
    assert!(output.contains("5"), "Expected '5' in: {}", output);
}

#[test]
fn test_option_with_string() {
    // Some with string literal
    let source = "μηνυμα τι «χαιρε» εστω.";
    let output = compile(source).unwrap();

    assert!(output.contains("Some"), "Expected 'Some' in: {}", output);
    assert!(output.contains("χαιρε"), "Expected 'χαιρε' in: {}", output);
}

#[test]
fn test_result_with_number_error() {
    // Err with number (unusual but valid)
    let source = "ξ σφαλμα δεκα εστω.";
    let output = compile(source).unwrap();

    assert!(output.contains("Err"), "Expected 'Err' in: {}", output);
    assert!(output.contains("10"), "Expected '10' in: {}", output);
}

#[test]
fn test_unwrap_preserves_value() {
    // Unwrap should generate .unwrap() call
    let source = "ξ τι πεντε εστω. ψ ξ! εστω.";
    let output = compile(source).unwrap();

    assert!(
        output.contains("expect"),
        "Expected 'expect' in: {}",
        output
    );
}

#[test]
fn test_option_in_print() {
    // Printing Option directly (not unwrapped)
    let source = "ξ τι πεντε εστω. ξ λεγε.";
    let output = compile(source).unwrap();

    assert!(output.contains("Some"), "Expected 'Some' in: {}", output);
    assert!(
        output.contains("println"),
        "Expected 'println' in: {}",
        output
    );
}

#[test]
fn test_separate_option_and_variable() {
    // Variables and Options are currently separate concepts
    // Wrapping variables in Option/Result requires explicit construction syntax (future feature)
    let source = r#"
        α πεντε εστω.
        β τι δεκα εστω.
    "#;
    let output = compile(source).unwrap();

    // Should have both a plain variable and an Option
    assert!(output.contains("5"), "Expected '5' in: {}", output);
    assert!(output.contains("Some"), "Expected 'Some' in: {}", output);
    assert!(output.contains("10"), "Expected '10' in: {}", output);
}

// ============================================================================
// Phase 9: Integration Tests - End-to-End Scenarios
// ============================================================================

#[test]
fn test_option_workflow() {
    // Complete Option workflow: create, check, unwrap
    let source = r#"
        α τι πεντε εστω.
        α λεγε.
        β α! εστω.
        β λεγε.
    "#;
    let output = compile(source).unwrap();

    // Should have Some(5)
    assert!(
        output.contains("Some") && output.contains("5"),
        "Expected Some(5) in: {}",
        output
    );
    // Should have two println calls
    let println_count = output.matches("println").count();
    assert!(
        println_count >= 2,
        "Expected at least 2 println calls, got {}",
        println_count
    );
    // Should have expect call
    assert!(output.contains("expect"), "Expected expect in: {}", output);
}

#[test]
fn test_result_workflow() {
    // Complete Result workflow: Ok and Err cases
    let source = r#"
        επιτυχη επιτυχια δεκα εστω.
        λαθος σφαλμα «προβλημα» εστω.
        επιτυχη λεγε.
        λαθος λεγε.
    "#;
    let output = compile(source).unwrap();

    // Should have Ok(10)
    assert!(
        output.contains("Ok") && output.contains("10"),
        "Expected Ok(10) in: {}",
        output
    );
    // Should have Err("problem")
    assert!(
        output.contains("Err") && output.contains("προβλημα"),
        "Expected Err with error message in: {}",
        output
    );
}

#[test]
fn test_mixed_option_result() {
    // Using both Option and Result in same program
    let source = r#"
        επιλογη τι πεντε εστω.
        αποτελεσμα επιτυχια δεκα εστω.
        επιλογη λεγε.
        αποτελεσμα λεγε.
    "#;
    let output = compile(source).unwrap();

    // Should have both Some and Ok
    assert!(output.contains("Some"), "Expected 'Some' in: {}", output);
    assert!(output.contains("Ok"), "Expected 'Ok' in: {}", output);
}

#[test]
fn test_none_handling() {
    // Working with None values
    let source = r#"
        κενον ουδεν εστω.
        κενον λεγε.
    "#;
    let output = compile(source).unwrap();

    // Should have None
    assert!(output.contains("None"), "Expected 'None' in: {}", output);
    assert!(
        output.contains("println"),
        "Expected println in: {}",
        output
    );
}

#[test]
fn test_multiple_unwraps_sequence() {
    // Multiple unwrap operations in sequence
    let source = r#"
        α τι πεντε εστω.
        β τι δεκα εστω.
        γ α! εστω.
        δ β! εστω.
    "#;
    let output = compile(source).unwrap();

    // Should have at least 2 unwrap calls
    let unwrap_count = output.matches("unwrap").count();
    assert!(
        unwrap_count >= 2,
        "Expected at least 2 unwrap calls, got {}: {}",
        unwrap_count,
        output
    );
}

#[test]
fn test_err_with_different_types() {
    // Err can wrap different value types
    let source = r#"
        α σφαλμα «μηνυμα» εστω.
        β σφαλμα πεντε εστω.
    "#;
    let output = compile(source).unwrap();

    // Should have two Err expressions
    let err_count = output.matches("Err").count();
    assert!(
        err_count >= 2,
        "Expected at least 2 Err calls, got {}: {}",
        err_count,
        output
    );
}

#[test]
fn test_some_with_different_types() {
    // Some can wrap different value types
    let source = r#"
        α τι «κειμενον» εστω.
        β τι πεντε εστω.
    "#;
    let output = compile(source).unwrap();

    // Should have two Some expressions
    let some_count = output.matches("Some").count();
    assert!(
        some_count >= 2,
        "Expected at least 2 Some calls, got {}: {}",
        some_count,
        output
    );
}

#[test]
fn test_realistic_error_handling() {
    // Realistic error handling pattern
    let source = r#"
        τιμη τι δεκα εστω.
        λαθος σφαλμα «ουκ ευρεθη» εστω.
        τιμη λεγε.
        λαθος λεγε.
    "#;
    let output = compile(source).unwrap();

    // Should compile successfully with both Some and Err
    assert!(output.contains("Some"), "Expected Some in: {}", output);
    assert!(output.contains("Err"), "Expected Err in: {}", output);
    // Should have Greek error message preserved
    assert!(
        output.contains("ευρεθη"),
        "Expected Greek error message in: {}",
        output
    );
}

// ============================================================================
// Phase 10: Propagation Operator (`;` after optative → `?`)
// ============================================================================

#[test]
fn test_propagation_operator_detection() {
    // RED: Test that `;` after an optative/Option expression generates `?`
    let source = "τιμη τι πεντε εστω; τιμη λεγε.";
    let output = compile(source);

    // Should compile successfully
    assert!(output.is_ok(), "Compilation failed: {:?}", output.err());
}

#[test]
fn test_propagation_generates_question_mark() {
    // RED: Verify `;` generates `?` operator in Rust
    let source = "τιμη τι πεντε εστω; τιμη λεγε.";
    let output = compile(source).unwrap();

    // Should contain the ? operator
    assert!(output.contains("?"), "Expected '?' operator in: {}", output);
}

#[test]
fn test_propagation_vs_unwrap() {
    // Propagation (`;`) should be different from unwrap (`!`)
    let unwrap_source = "α τι πεντε εστω. β α! εστω.";
    let propagate_source = "α τι πεντε εστω; β α εστω.";

    let unwrap_output = compile(unwrap_source).unwrap();
    let propagate_output = compile(propagate_source).unwrap();

    // Unwrap should have .expect()
    assert!(
        unwrap_output.contains("expect"),
        "Expected expect in: {}",
        unwrap_output
    );

    // Propagation should have ?
    assert!(
        propagate_output.contains("?"),
        "Expected ? in: {}",
        propagate_output
    );
}

#[test]
fn test_propagation_with_result() {
    // RED: Propagation works with Result types
    let source = "αποτελεσμα επιτυχια δεκα εστω; αποτελεσμα λεγε.";
    let output = compile(source).unwrap();

    assert!(output.contains("Ok"), "Expected Ok in: {}", output);
    assert!(output.contains("?"), "Expected ? in: {}", output);
}

#[test]
fn test_chained_propagation() {
    // RED: Multiple propagation operators in sequence
    let source = r#"
        α τι πεντε εστω;
        β τι δεκα εστω;
        α λεγε.
        β λεγε.
    "#;
    let output = compile(source).unwrap();

    // Should have at least 2 ? operators
    let question_count = output.matches('?').count();
    assert!(
        question_count >= 2,
        "Expected at least 2 '?' operators, got {}: {}",
        question_count,
        output
    );
}

#[test]
fn test_propagation_early_return() {
    // Propagation should enable early return pattern
    let source = r#"
        α τι πεντε εστω;
        β α εστω.
    "#;
    let output = compile(source).unwrap();

    // The pattern should propagate None/Err upward
    assert!(output.contains("?"), "Expected ? operator in: {}", output);
}

#[test]
fn test_statement_end_with_semicolon() {
    // Regular statement end (.) vs propagation (;)
    let regular = "α τι πεντε εστω. α λεγε.";
    let propagate = "α τι πεντε εστω; α λεγε.";

    let regular_output = compile(regular).unwrap();
    let propagate_output = compile(propagate).unwrap();

    // Regular should NOT have ?
    assert!(
        !regular_output.contains("?"),
        "Should not have ? in regular statement"
    );

    // Propagate should have ?
    assert!(propagate_output.contains("?"), "Expected ? in propagation");
}

#[test]
fn test_propagation_in_workflow() {
    // Realistic workflow with propagation
    let source = r#"
        τιμη τι πεντε εστω;
        αποτελεσμα επιτυχια τιμη εστω.
    "#;
    let output = compile(source);

    assert!(output.is_ok(), "Compilation failed: {:?}", output.err());
}
