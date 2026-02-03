//! Tests for AssemblyError variants to ensure code coverage

use glossa::errors::AssemblyError;

#[test]
fn test_statement_too_long_display() {
    let err = AssemblyError::StatementTooLong {
        count: 1001,
        limit: 1000,
    };
    let display = format!("{}", err);
    assert!(display.contains("Πρότασις λίαν μακρά"));
    assert!(display.contains("1001"));
    assert!(display.contains("1000"));
}
