use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_undefined_variable_in_print_subject() {
    // "logos lege" -> undefined variable "logos" (Subject)
    let code = "λογος λεγε.";
    let ast = parse(code).expect("Parsing failed");
    let result = analyze_program(&ast);

    if let Ok(program) = &result {
        println!("Analysis result: {:?}", program.statements);
        println!("Scope: {:?}", program.scope);
    }
    assert!(result.is_err(), "Should fail semantic analysis");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown variable"), "Error should be about unknown variable: {}", err_msg);
    assert!(err_msg.contains("λογος"), "Error should mention the variable name");
}

#[test]
fn test_undefined_variable_in_print_object() {
    // "xi onoma lege" -> xi is subject (defined), onoma is object (undefined)
    // We define xi first to fill the subject slot.
    let code = "ξ 5 εστω. ξ ονομα λεγε.";
    let ast = parse(code).expect("Parsing failed");
    let result = analyze_program(&ast);

    if let Ok(program) = &result {
        println!("Analysis result: {:?}", program.statements);
        println!("Scope: {:?}", program.scope);
    }
    assert!(result.is_err(), "Should fail semantic analysis");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown variable"), "Error should be about unknown variable: {}", err_msg);
    assert!(err_msg.contains("ονομα"), "Error should mention the variable name");
}

#[test]
fn test_codegen_print_primitive() {
    let code = "ξ 5 εστω. ξ λεγε.";
    let ast = parse(code).expect("Parsing failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let rust = generate_rust(&analyzed);

    // Should use Display formatting "{}"
    assert!(rust.contains("println ! (\"{}\" , g_x)"), "Should use Display formatting for numbers: {}", rust);
}

#[test]
fn test_codegen_print_struct() {
    let code = "
    ειδος Point οριζειν { x αριθμου. }.
    π νεον Point 5 εστω.
    π λεγε.
    ";
    let ast = parse(code).expect("Parsing failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let rust = generate_rust(&analyzed);

    // Should use Debug formatting "{:?}"
    assert!(rust.contains("println ! (\"{:?}\" , g_p)"), "Should use Debug formatting for structs: {}", rust);
}

#[test]
fn test_codegen_print_mixed() {
    let code = "
    ειδος Point οριζειν { x αριθμου. }.
    π νεον Point 5 εστω.
    ξ 10 εστω.
    π ξ λεγε.
    ";
    let ast = parse(code).expect("Parsing failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let rust = generate_rust(&analyzed);

    // For multiple args, it generates format! macro calls joined by space
    // format!("{}", g_x) for number
    // format!("{:?}", g_p) for struct
    println!("Generated Rust: {}", rust);
    assert!(rust.contains("format ! (\"{:?}\" , g_p)"), "Should format struct with Debug");
    // ξ (xi) transliterates to g_x, not hex encoded
    assert!(rust.contains("format ! (\"{}\" , g_x)"), "Should format number with Display");
}
