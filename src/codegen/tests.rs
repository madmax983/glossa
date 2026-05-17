#![allow(unused_imports)]
use super::*;
use crate::parser::parse;
use crate::semantic::analyze_program;
use smol_str::SmolStr;

// --- Utils Tests ---

#[test]
fn test_sanitize_greek_letter() {
    assert_eq!(sanitize_name("ξ"), "g__u3be_");
    assert_eq!(sanitize_name("α"), "g__u3b1_");
    assert_eq!(sanitize_name("ω"), "g__u3c9_");
}

#[test]
fn test_transliterate() {
    // All Greek characters are now hex encoded to avoid ASCII collisions
    // χ (chi) -> _u3c7_
    // ρ -> _u3c1_
    // η -> _u3b7_
    // ...
    assert_eq!(
        transliterate("χρηστος"),
        "_u3c7__u3c1__u3b7__u3c3__u3c4__u3bf__u3c2_"
    );
    assert_eq!(transliterate("λογος"), "_u3bb__u3bf__u3b3__u3bf__u3c2_");
    assert_eq!(
        transliterate("φιλοσοφια"),
        "_u3c6__u3b9__u3bb__u3bf__u3c3__u3bf__u3c6__u3b9__u3b1_"
    );
}

#[test]
fn test_transliterate_unique() {
    // Test that different invalid characters produce different outputs
    let koppa = "ϟ";
    let stigma = "ϛ";

    let t_koppa = transliterate(koppa);
    let t_stigma = transliterate(stigma);

    assert_ne!(
        t_koppa, t_stigma,
        "Different invalid chars should not collide"
    );
    assert!(t_koppa.contains("_u3df_")); // Koppa is 0x3DF
    assert!(t_stigma.contains("_u3db_")); // Stigma is 0x3DB
}

#[test]
fn test_transliterate_mixed_valid_invalid() {
    // Test mixing valid and invalid characters
    // α -> _u3b1_
    // ϟ -> _u3df_
    // β -> _u3b2_
    let input = "αϟβ";
    let output = transliterate(input);
    assert_eq!(output, "_u3b1__u3df__u3b2_");
}

#[test]
fn test_sanitize_keywords_and_prefix() {
    // Test that keywords are safe (by prefixing)
    // If "if" stays "if", it's invalid Rust
    assert_eq!(sanitize_name("if"), "g_if");
    assert_eq!(sanitize_name("fn"), "g_fn");

    // Test that regular identifiers are prefixed
    // This ensures a unique namespace for user variables
    assert_eq!(sanitize_name("x"), "g_x");
    assert_eq!(sanitize_name("foo"), "g_foo");
}

#[test]
fn test_sanitize_edge_cases() {
    // Test empty strings
    assert_eq!(sanitize_name(""), "g__var_empty");
    assert_eq!(transliterate(""), "_var_empty");

    // Test numeric start
    // sanitize_name adds "g_", so "123" becomes "g_123"
    assert_eq!(sanitize_name("123"), "g_123");

    // transliterate handles numeric start by prepending "_"
    assert_eq!(transliterate("123"), "_123");

    // Test numeric start mixed
    assert_eq!(transliterate("1a"), "_1a");
}

// --- Types Tests ---

#[test]
fn test_basic_types() {
    assert_eq!(to_rust_type(&GlossaType::Number), "i64");
    assert_eq!(to_rust_type(&GlossaType::String), "String");
    assert_eq!(to_rust_type(&GlossaType::Boolean), "bool");
    assert_eq!(to_rust_type(&GlossaType::Unit), "()");
    assert_eq!(to_rust_type(&GlossaType::Unknown), "_");
}

#[test]
fn test_container_types() {
    assert_eq!(
        to_rust_type(&GlossaType::List(Box::new(GlossaType::Number))),
        "Vec<i64>"
    );
    assert_eq!(
        to_rust_type(&GlossaType::Set(Box::new(GlossaType::String))),
        "HashSet<String>"
    );
    assert_eq!(
        to_rust_type(&GlossaType::Map(
            Box::new(GlossaType::String),
            Box::new(GlossaType::Number)
        )),
        "HashMap<String, i64>"
    );
    assert_eq!(
        to_rust_type(&GlossaType::Option(Box::new(GlossaType::Number))),
        "Option<i64>"
    );
    assert_eq!(
        to_rust_type(&GlossaType::Result(
            Box::new(GlossaType::Number),
            Box::new(GlossaType::String)
        )),
        "Result<i64, String>"
    );
}

#[test]
fn test_struct_type() {
    // Use normalized name as per compiler convention
    let ty = GlossaType::Struct {
        name: SmolStr::new("χρηστης"),
        gender: crate::morphology::Gender::Masculine,
        fields: vec![],
    };
    // Sanitize: χρηστης -> g__u3c7__u3c1__u3b7__u3c3__u3c4__u3b7__u3c2_
    // Capitalize: g_... -> G_...
    assert_eq!(
        to_rust_type(&ty),
        "G__u3c7__u3c1__u3b7__u3c3__u3c4__u3b7__u3c2_"
    );
}

// --- Rust Codegen Tests ---

fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    generate_rust(&analyzed)
}

#[test]
fn test_generate_hello() {
    let code = compile("«χαῖρε» λέγε.");
    // quote! generates `println !` with space
    assert!(code.contains("println"), "Expected println in: {}", code);
    assert!(code.contains("χαῖρε"));
}

#[test]
fn test_generate_binding() {
    let code = compile("ξ πέντε ἔστω.");
    // Variables are now prefixed with g_ and hex encoded
    assert!(code.contains("let g__u3be_"));
    assert!(code.contains("5"));
}

#[test]
fn test_generate_number() {
    let code = compile("42 λέγε.");
    assert!(code.contains("println"), "Expected println in: {}", code);
    assert!(code.contains("42"));
}

#[test]
fn test_generate_full_program() {
    let code = compile("ξ πέντε ἔστω. ξ λέγε.");
    // Variables are now prefixed with g_ and hex encoded
    assert!(
        code.contains("let g__u3be_ = 5"),
        "Expected binding in: {}",
        code
    );
    assert!(code.contains("println"), "Expected println in: {}", code);
}

#[test]
fn test_generate_statement_code() {
    let ast = parse("«χαῖρε» λέγε.").unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let stmt = &analyzed.statements[0];
    let code = generate_statement_code(stmt);
    assert!(code.contains("println"));
    assert!(code.contains("χαῖρε"));
}

#[test]
fn test_generate_checked_op() {
    let left = quote! { a };
    let right = quote! { b };
    let code = generate_checked_op(left, right, "checked_add", "arithmetic overflow").to_string();
    assert!(code.contains("checked_add"));
    assert!(code.contains("expect"));
    assert!(code.contains("arithmetic overflow"));
}

#[test]
fn test_generate_unary_op_neg_checked() {
    let expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    };
    let code = generate_unary_op(UnaryOp::Neg, &expr).to_string();
    assert!(code.contains("checked_neg"));
    assert!(code.contains("expect"));
    assert!(code.contains("arithmetic overflow"));
}

#[test]
fn test_generate_collection_index_bounds_check() {
    let array = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable("arr".into()),
        glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
    };
    let index = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    };
    let code = generate_collection_index(&array, &index).to_string();
    assert!(code.contains("try_from"));
    assert!(code.contains("expect (\"index out of bounds: too large\")"));
    assert!(code.contains("index out of bounds: negative index"));
    assert!(code.contains("expect (\"index out of bounds: index too large\")"));
}

#[test]
fn test_generate_control_unwrap() {
    let expr = AnalyzedExpr {
        expr: AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(42),
            glossa_type: GlossaType::Number,
        })),
        glossa_type: GlossaType::Number,
    };

    let code = generate_expr(&expr).to_string();
    assert!(code.contains("expect"));
    assert!(code.contains("attempted to unwrap an empty value"));
    assert!(code.contains("42"));
}

#[test]
fn test_generate_unreachable_operators() {
    // Manually trigger fallback operators like Le/Ge that aren't parsed yet
    let left = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(5),
        glossa_type: GlossaType::Number,
    };
    let right = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(10),
        glossa_type: GlossaType::Number,
    };

    // Test Ge (Greater or Equal)
    let op_ge = BinaryOp::Ge;
    let tokens_ge = generate_bin_op(op_ge, &left, &right);
    let code_ge = tokens_ge.to_string();
    assert!(code_ge.contains(">="));

    // Test Le (Less or Equal)
    let op_le = BinaryOp::Le;
    let tokens_le = generate_bin_op(op_le, &left, &right);
    let code_le = tokens_le.to_string();
    assert!(code_le.contains("<="));
}

mod refactor_tests {
    use super::*;
    use smol_str::SmolStr;

    #[test]
    fn test_generate_trait_parts_manual() {
        // Manual construction of a trait method
        let method = AnalyzedMethod {
            name: SmolStr::new("test_method"),
            params: vec![
                (SmolStr::new("self"), GlossaType::Unknown), // Self param
                (SmolStr::new("arg1"), GlossaType::Number),
            ],
            body: None,
            return_type: Some(GlossaType::Boolean),
        };

        let parts = generate_trait_method_parts(&method);

        assert_eq!(parts.name.to_string(), "g_test_method");

        // Check params
        let params_str: Vec<String> = parts.params.iter().map(|t| t.to_string()).collect();
        assert_eq!(params_str.len(), 2);
        assert_eq!(params_str[0], "& self");
        assert!(params_str[1].contains("g_arg1"));
        assert!(params_str[1].contains("i64"));

        // Check return type
        assert!(parts.return_type.is_some());
        assert_eq!(parts.return_type.unwrap().to_string(), "bool");
    }

    #[test]
    fn test_generate_checked_arithmetic() {
        let left = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(10),
            glossa_type: GlossaType::Number,
        };
        let right = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(20),
            glossa_type: GlossaType::Number,
        };

        let op_add = BinaryOp::Add;
        let tokens = generate_bin_op(op_add, &left, &right);
        let code = tokens.to_string();

        assert!(code.contains("checked_add"));
        assert!(code.contains("expect"));
    }

    #[test]
    fn test_generate_rust_large_program() {
        // Manually construct an AnalyzedProgram with numerous statements
        // to verify pre-allocation logic doesn't fail under load.
        let mut statements = Vec::with_capacity(100);

        for i in 0..20 {
            statements.push(AnalyzedStatement::FunctionDef {
                name: SmolStr::new(format!("func_{}", i)),
                params: vec![],
                body: vec![],
                return_type: None,
            });
            statements.push(AnalyzedStatement::TypeDefinition {
                name: SmolStr::new(format!("type_{}", i)),
                fields: vec![],
            });
            statements.push(AnalyzedStatement::TraitDefinition {
                name: SmolStr::new(format!("trait_{}", i)),
                methods: vec![],
            });
            statements.push(AnalyzedStatement::TestDeclaration {
                name: format!("test_{}", i),
                body: vec![],
            });
            statements.push(AnalyzedStatement::Return {
                value: Some(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(i as i64),
                    glossa_type: GlossaType::Number,
                })),
            });
        }

        let program = AnalyzedProgram {
            statements,
            scope: crate::semantic::Scope::new(),
        };
        let result = generate_rust(&program);

        // Check that it didn't panic and produced some code
        assert!(!result.is_empty());
        assert!(result.contains("fn g_func_0"));
        assert!(result.contains("struct G_type_0"));
        assert!(result.contains("trait G_trait_0"));
    }
}
