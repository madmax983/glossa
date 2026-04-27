#![allow(missing_docs)]
use glossa::tools::runner::analyze_source;

#[test]
fn test_killswitch_invalid_file() {
    let source = "«test» λέγε.";
    let result = analyze_source(source);
    assert!(result.is_ok());
}
