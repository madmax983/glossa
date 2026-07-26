#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::parser::parse_greek_numeral;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn fuzz_numerals(s in "[\\u0370-\\u03FF]*") {
        // Attempt to parse random Greek strings.
        // We don't care about the result (Ok/Err), only that it doesn't panic.
        let _ = parse_greek_numeral(&s);
    }

    #[test]
    fn fuzz_parser(s in "\\PC*") {
        // Fuzz the entire parser with random strings.
        // This checks for panics in the grammar or builder.
        let _ = parse(&s);
    }
}

#[test]
fn test_huge_numeral_overflow_attempt() {
    // Attempt to overflow i64 with a massive string of 900s (ϡ)
    // Each ϡ is 900.
    // i64::MAX is ~9e18.
    // We need ~1e16 chars to overflow.
    // We can't allocate that. But we can verify it handles a large-ish string gracefully.
    // And if we ever switch to i32, this would catch it.
    let huge_string = "ϡ".repeat(100_000); // 900 * 100,000 = 90,000,000
    let res = parse_greek_numeral(&huge_string);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 90_000_000);
}

#[test]
fn havoc_codegen_stack_overflow() {
    if std::env::var("HAVOC_TRIGGER").is_ok() {
        let mut expr = glossa::semantic::AnalyzedExpr {
            expr: glossa::semantic::AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: glossa::semantic::GlossaType::Boolean,
        };
        for _ in 0..100000 {
            expr = glossa::semantic::AnalyzedExpr {
                expr: glossa::semantic::AnalyzedExprKind::UnaryOp {
                    op: glossa::morphology::UnaryOp::Not,
                    operand: Box::new(expr)
                },
                glossa_type: glossa::semantic::GlossaType::Boolean,
            };
        }
        let ast = glossa::semantic::AnalyzedProgram {
            statements: vec![glossa::semantic::AnalyzedStatement::Return {
                value: Some(Box::new(expr)),
            }],
            scope: glossa::semantic::Scope::new(),
        };
        glossa::codegen::generate_rust(&ast);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let output = std::process::Command::new(exe)
        .env("HAVOC_TRIGGER", "1")
        .arg("--nocapture")
        .arg("havoc_codegen_stack_overflow")
        .output()
        .unwrap();

    assert!(!output.status.success(), "Expected to crash");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stack overflow"), "Expected stack overflow in stderr");
}
