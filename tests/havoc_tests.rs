use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
#[should_panic]
fn test_havoc_deep_nesting_panic() {
    let mut s = String::new();
    for _ in 0..10000 {
        s.push_str("εἰ ἀληθὲς ᾖ, {\n");
    }
    s.push_str("«τέλος» λέγε.\n");
    for _ in 0..10000 {
        s.push_str("}\n");
    }
    let ast = parse(&s).unwrap();
    let _ = analyze_program(&ast);
}

#[test]
#[should_panic]
fn test_havoc_codegen_stack_overflow() {
    let mut s = String::new();
    for _ in 0..10000 {
        s.push_str("εἰ ἀληθὲς ᾖ, {\n");
    }
    s.push_str("«τέλος» λέγε.\n");
    for _ in 0..10000 {
        s.push_str("}\n");
    }
    let ast = parse(&s).unwrap();
    if let Ok(program) = analyze_program(&ast) {
        let _ = glossa::codegen::generate_rust(&program);
    } else {
        panic!("analysis failed");
    }
}

#[test]
#[should_panic]
fn test_havoc_lexer_stack_overflow() {
    let mut s = String::new();
    for _ in 0..100000 {
        s.push('(');
    }
    for _ in 0..100000 {
        s.push(')');
    }
    let ast = parse(&s).unwrap();
    let _ = analyze_program(&ast);
}
