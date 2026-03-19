use glossa::tools::runner::{check_file, build_file, highlight_file, bard_file};
use glossa::tools::tester::run_tests;
use std::path::PathBuf;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ui_coverage_early_exits() {
    let non_existent = PathBuf::from("does_not_exist.gl");

    // Test check_file missing file (hits the early load_source? error)
    let _ = check_file(&non_existent);
    let _ = build_file(&non_existent, None);
    let _ = highlight_file(&non_existent);
    let _ = bard_file(&non_existent);
    let _ = run_tests(&non_existent);
}

#[test]
fn test_ui_coverage_syntax_error() {
    let dir = tempdir().unwrap();
    let bad_syntax = dir.path().join("bad.gl");
    fs::write(&bad_syntax, "invalid syntax!!!").unwrap();

    let _ = check_file(&bad_syntax);
    let _ = build_file(&bad_syntax, None);
    let _ = highlight_file(&bad_syntax);
    let _ = bard_file(&bad_syntax);
    let _ = run_tests(&bad_syntax);
}

#[test]
fn test_ui_coverage_semantic_error() {
    let dir = tempdir().unwrap();
    let bad_semantic = dir.path().join("bad2.gl");
    // Invalid semantics, e.g. undefined variable
    fs::write(&bad_semantic, "unknown_var λέγε.").unwrap();

    let _ = check_file(&bad_semantic);
    let _ = build_file(&bad_semantic, None);
    // highlight ignores semantics mostly but doesn't fail,
    // run_tests catches semantic errors explicitly.
    let _ = run_tests(&bad_semantic);
}
