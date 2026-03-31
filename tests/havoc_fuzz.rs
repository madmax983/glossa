#![allow(missing_docs)]
use glossa::parser::parse;
use glossa::semantic::analyze_program;
use proptest::prelude::*;

proptest! {
    // Fuzz the parser with random Unicode strings
    // This catches panics in the parser or lexer
    #[test]
    fn fuzz_parser(s in "\\PC*") {
        // We don't care if it errors, only if it panics
        let _ = parse(&s);
    }

    // Fuzz the parser with deeper structure-like strings
    #[test]
    fn fuzz_parser_structured(s in "[a-zA-Z0-9_\\s\\.\\(\\)\\{\\}\\[\\]«»]+") {
        let _ = parse(&s);
    }

    // Fuzz the semantic analyzer with valid ASTs (if parser succeeds)
    // We can't easily generate valid ASTs directly because of private fields or complex invariants
    // But we can generate random valid-ish source code and feed it
    #[test]
    fn fuzz_semantic_validish_source(
        // Generate random Greek words
        verb in "λέγε|γράφει|τρέχει",
        noun in "άνθρωπος|λόγον|θεός|φως",
        literal in "«.*»|[0-9]+",
    ) {
        let source = format!("{} {} {}.", noun, literal, verb);
        if let Ok(ast) = parse(&source) {
            // If parsed, it should analyze without panic
            let _ = analyze_program(&ast);
        }
    }
}
