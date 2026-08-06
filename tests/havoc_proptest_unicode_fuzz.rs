//! Havoc fuzzing tests for handling arbitrary unicode input.
//!
//! This module uses property-based testing to throw random Unicode characters
//! at the parser and semantic analyzer, ensuring they don't panic on malformed input.

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
