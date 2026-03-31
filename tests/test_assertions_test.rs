#![allow(missing_docs)]
//! Integration tests for δεῖ and ἰσοῦται assertion transpilation

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_assert_containment_compiles() {
    let source = r#"
δοκιμή «HashMap containment».
    χ νέον χάρτης ἔστω.
    χ 2 0 τίθησι.
    2 ἐν χ δεῖ.
τέλος.
"#;

    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Semantic analysis failed");
    let rust_code = generate_rust(&analyzed);

    let rust_str = rust_code.to_string();

    // Should generate assert!(chi.contains_key(&2i64))
    assert!(rust_str.contains("assert !"), "Missing assert! macro");
    assert!(
        rust_str.contains("contains_key"),
        "Missing contains_key call"
    );
    assert!(rust_str.contains("# [test]"), "Missing #[test] attribute");
}

#[test]
fn test_assert_eq_compiles() {
    let source = r#"
δοκιμή «equality check».
    κ 5 ἔστω.
    κ 5 ἰσοῦται.
τέλος.
"#;

    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Semantic analysis failed");
    let rust_code = generate_rust(&analyzed);

    let rust_str = rust_code.to_string();

    eprintln!("Generated code:\n{}", rust_str);

    // Should generate assert_eq !(k, 5i64)
    assert!(
        rust_str.contains("assert_eq !"),
        "Missing assert_eq ! macro. Got:\n{}",
        rust_str
    );
    assert!(rust_str.contains("k"), "Missing variable reference");
    assert!(rust_str.contains("5i64"), "Missing literal value");
    assert!(rust_str.contains("# [test]"), "Missing #[test] attribute");
}

#[test]
fn test_multiple_assertions_in_one_test() {
    let source = r#"
δοκιμή «multiple assertions».
    μετά κ 0 ἔστω.
    κ 0 ἰσοῦται.
    κ 1 γίγνεται.
    κ 1 ἰσοῦται.
τέλος.
"#;

    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Semantic analysis failed");
    let rust_code = generate_rust(&analyzed);

    let rust_str = rust_code.to_string();

    // Should have two assert_eq ! calls
    let assert_count = rust_str.matches("assert_eq !").count();
    assert_eq!(assert_count, 2, "Expected 2 assert_eq ! calls");
}

#[test]
fn test_test_function_name_sanitization() {
    let source = r#"
δοκιμή «Test with Spaces and-Dashes».
    ξ 5 ἔστω.
τέλος.
"#;

    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Semantic analysis failed");
    let rust_code = generate_rust(&analyzed);

    let rust_str = rust_code.to_string();

    // Function name should be sanitized to test_test_with_spaces_and_dashes
    assert!(
        rust_str.contains("fn test_test_with_spaces_and_dashes"),
        "Function name not properly sanitized: {}",
        rust_str
    );
}

#[test]
fn test_hashset_assertion() {
    let source = r#"
δοκιμή «HashSet contains».
    σ νέον σύνολον ἔστω.
    σ 1 τίθησι.
    1 ἐν σ δεῖ.
τέλος.
"#;

    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Semantic analysis failed");
    let rust_code = generate_rust(&analyzed);

    let rust_str = rust_code.to_string();

    assert!(rust_str.contains("assert !"), "Missing assert! macro");
    assert!(rust_str.contains("contains"), "Missing contains call");
}

#[test]
fn test_empty_test_body() {
    let source = r#"
δοκιμή «empty test».
τέλος.
"#;

    let ast = parse(source).expect("Parse failed");
    let analyzed = analyze_program(&ast).expect("Semantic analysis failed");
    let rust_code = generate_rust(&analyzed);

    let rust_str = rust_code.to_string();

    // Should generate an empty test function
    assert!(rust_str.contains("# [test]"), "Missing #[test] attribute");
    assert!(rust_str.contains("fn test_empty_test"), "Missing function");
}
