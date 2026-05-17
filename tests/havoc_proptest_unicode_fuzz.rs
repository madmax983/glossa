//! Tests for fuzzed unicode handling in the parser and semantic analyzer.
//!
//! This module uses `proptest` to throw random non-ASCII characters at the compiler
//! to ensure it does not panic or crash when handling invalid unicode input.

use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_weird_unicode_fuzz(s in "\\PC*") {
        if let Ok(ast) = parse(&s) {
            let _ = analyze_program(&ast);
        }
    }
}
