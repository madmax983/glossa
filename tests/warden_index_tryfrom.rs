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
fn test_warden_index_tryfrom_in_scope() {
    let array_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::ArrayLiteral(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(10),
            glossa_type: GlossaType::Number,
        }]),
        glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
    };

    let index_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(-1),
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
    // generate_rust_file output is not inside a block but top level
    f.write_all(format!("fn main() {{ {} }}", code).as_bytes())
        .unwrap();

    let rustc_cmd = std::env::var("GLOSSA_RUSTC_CMD").unwrap_or("rustc".to_string());

    let output = Command::new(rustc_cmd)
        .arg(&file_path)
        .arg("--out-dir")
        .arg(dir.path())
        .output()
        .expect("Failed to execute rustc");

    assert!(
        output.status.success(),
        "Generated code failed to compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
