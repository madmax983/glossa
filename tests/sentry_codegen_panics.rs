#![allow(missing_docs)]
use glossa::codegen::generate_rust_file;
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_sentry_bounds_check_execution() {
    let cases = vec![
        (-1, "index out of bounds: negative index"),
        (999, "index out of bounds: index too large"),
    ];

    for (index_val, expected_err) in cases {
        let array_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::ArrayLiteral(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(10),
                glossa_type: GlossaType::Number,
            }]),
            glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
        };

        let index_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(index_val),
            glossa_type: GlossaType::Number,
        };

        let access_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::IndexAccess {
                array: Box::new(array_expr),
                index: Box::new(index_expr),
            },
            glossa_type: GlossaType::Number,
        };

        let stmt = AnalyzedStatement::Expression(vec![access_expr]);

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let code = generate_rust_file(&program);

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(code.as_bytes()).unwrap();

        let rustc_cmd = std::env::var("GLOSSA_RUSTC_CMD").unwrap_or("rustc".to_string());

        let build_output = Command::new(rustc_cmd)
            .arg(&file_path)
            .arg("--out-dir")
            .arg(dir.path())
            .output()
            .expect("Failed to execute rustc");

        assert!(
            build_output.status.success(),
            "Generated code failed to compile: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );

        let binary_path = dir.path().join("main");
        let run_output = Command::new(binary_path)
            .output()
            .expect("Failed to run binary");

        assert!(
            !run_output.status.success(),
            "Binary should have panicked for {}! status: {}, stdout: {}, stderr: {}",
            expected_err,
            run_output.status,
            String::from_utf8_lossy(&run_output.stdout),
            String::from_utf8_lossy(&run_output.stderr)
        );

        let stderr = String::from_utf8_lossy(&run_output.stderr);
        assert!(
            stderr.contains(expected_err)
                || stderr.contains("Δείκτης ἐκτὸς ὁρίων")
                || stderr.contains("Index out of bounds"),
            "Expected stderr to contain '{}', but got: {}",
            expected_err,
            stderr
        );
    }
}

#[test]
fn test_sentry_unwrap_execution() {
    let unwrap_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("g_none_var".into()),
            glossa_type: GlossaType::Number,
        })),
        glossa_type: GlossaType::Number,
    };

    let stmt = AnalyzedStatement::Expression(vec![unwrap_expr]);

    let scope = Scope::new();
    // In Rust, None for numbers is an Option<i64>
    // We'll just generate the code and check if the unwrap panic happens.
    // However, to make it compile, `none_var` needs to be defined in rust scope.
    // So we'll wrap it in a function.
    let program = AnalyzedProgram {
        statements: vec![stmt],
        scope,
    };

    let code = generate_rust_file(&program);

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("main.rs");
    let mut f = File::create(&file_path).unwrap();

    // Create a rust context where none_var is an Option that is None
    let mut code = code;
    code = code.replace("#![allow(non_snake_case, unused_imports)]", "");
    code = code.replace("use std::convert::TryFrom;", "");

    code = code.replace(
        "fn main () {",
        "fn main () {\nlet g_g_none_var: Option<i64> = None;",
    );
    f.write_all(code.as_bytes()).unwrap();

    let rustc_cmd = std::env::var("GLOSSA_RUSTC_CMD").unwrap_or("rustc".to_string());

    let build_output = Command::new(rustc_cmd)
        .arg(&file_path)
        .arg("--out-dir")
        .arg(dir.path())
        .output()
        .expect("Failed to execute rustc");

    assert!(
        build_output.status.success(),
        "Generated code failed to compile: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let binary_path = dir.path().join("main");
    let run_output = Command::new(binary_path)
        .output()
        .expect("Failed to run binary");

    assert!(
        !run_output.status.success(),
        "Binary should have panicked for unwrap empty value!"
    );

    let stderr = String::from_utf8_lossy(&run_output.stderr);
    let expected_err = "attempted to unwrap an empty value";
    assert!(
        stderr.contains(expected_err) || stderr.contains("Σφάλμα ἐκτελέσεως"),
        "Expected stderr to contain '{}', but got: {}",
        expected_err,
        stderr
    );
}
