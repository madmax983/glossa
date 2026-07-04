#![cfg(feature = "nova")]

use glossa::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType, AnalyzedStatement};
use glossa::tools::gnomon::{GnomonVisitor, run_gnomon};
use smol_str::SmolStr;

fn dummy_expr() -> Box<AnalyzedExpr> {
    Box::new(AnalyzedExpr {
        expr: AnalyzedExprKind::BooleanLiteral(true),
        glossa_type: GlossaType::Boolean,
    })
}

#[test]
fn test_gnomon_if_statement() {
    let mut visitor = GnomonVisitor::new();
    let inner_loop = AnalyzedStatement::While {
        condition: dummy_expr(),
        body: vec![],
    };
    let stmt = AnalyzedStatement::If {
        condition: dummy_expr(),
        then_body: vec![inner_loop.clone()],
        else_body: Some(vec![inner_loop]),
    };
    visitor.visit_statement(&stmt);
    assert_eq!(visitor.max_depth, 1);
}

#[test]
fn test_gnomon_match_statement() {
    let mut visitor = GnomonVisitor::new();
    let inner_loop = AnalyzedStatement::While {
        condition: dummy_expr(),
        body: vec![],
    };
    let stmt = AnalyzedStatement::Match {
        scrutinee: dummy_expr(),
        arms: vec![(*dummy_expr(), vec![inner_loop])],
    };
    visitor.visit_statement(&stmt);
    assert_eq!(visitor.max_depth, 1);
}

#[test]
fn test_gnomon_function_def() {
    let mut visitor = GnomonVisitor::new();
    let inner_loop = AnalyzedStatement::While {
        condition: dummy_expr(),
        body: vec![],
    };
    let stmt = AnalyzedStatement::FunctionDef {
        name: SmolStr::new("func"),
        params: vec![],
        return_type: None,
        body: vec![inner_loop],
    };
    visitor.visit_statement(&stmt);
    assert_eq!(visitor.max_depth, 1);
}

#[test]
fn test_gnomon_test_declaration() {
    let mut visitor = GnomonVisitor::new();
    let inner_loop = AnalyzedStatement::While {
        condition: dummy_expr(),
        body: vec![],
    };
    let stmt = AnalyzedStatement::TestDeclaration {
        name: SmolStr::new("test").to_string(),
        body: vec![inner_loop],
    };
    visitor.visit_statement(&stmt);
    assert_eq!(visitor.max_depth, 1);
}

#[test]
fn test_run_gnomon_success() {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "«χαῖρε» λέγε.").unwrap();
    let result = run_gnomon(file.path());
    assert!(result.is_ok());
}

#[test]
fn test_run_gnomon_file_not_found() {
    let result = run_gnomon(std::path::Path::new("does_not_exist_file.γλ"));
    assert!(result.is_err());
}

#[test]
fn test_run_gnomon_syntax_error() {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "invalid syntax").unwrap();
    let result = run_gnomon(file.path());
    assert!(result.is_err());
}

#[test]
fn test_run_gnomon_complexity_n2() {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "ἕως 1 ἴσον 1, ἕως 1 ἴσον 1, «test» λέγε.  ").unwrap();
    let result = run_gnomon(file.path());
    assert!(result.is_ok());
}

#[test]
fn test_run_gnomon_complexity_n3() {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().unwrap();
    writeln!(file, "ἕως 1 ἴσον 1, ἕως 1 ἴσον 1, ἕως 1 ἴσον 1, «test» λέγε.   ").unwrap();
    let result = run_gnomon(file.path());
    assert!(result.is_ok());
}
