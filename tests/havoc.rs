use proptest::prelude::*;
use glossa::parser::numerals::parse_greek_numeral;
use loom::thread;
use glossa::morphology::lexicon::{lookup, is_verb};
use glossa::morphology::participle::analyze_participle;

// 1. Loom Concurrency Torture
// While Lexicon uses LazyLock, loom thread interleaving can expose if any implicit mutations occur.
// We wrap it in Loom's execution model.
#[test]
fn test_concurrency_torture_lexicon() {
    loom::model(|| {
        let t1 = thread::spawn(|| {
            lookup("λεγε");
        });
        let t2 = thread::spawn(|| {
            is_verb("λεγε");
        });
        let t3 = thread::spawn(|| {
            analyze_participle("γραφων");
        });

        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
    });
}

// 2. Fuzzing API Layers & Parsing (Proptest)
proptest! {
    // Tests if any random Unicode noise crashes the parser.
    #[test]
    fn fuzz_parser_api_layer(s in "\\PC*") {
        let _ = glossa::parser::parse(&s);
    }

    // Test if arbitrary length integers crash numeral parsing
    #[test]
    fn proptest_greek_numeral_edge_cases(
        keraia_count in 0..10_000usize
    ) {
        let mut numeral = String::new();
        // Lower keraia multiplier spam
        for _ in 0..keraia_count {
            numeral.push('\u{0375}');
        }
        numeral.push('α'); // append 1

        // Let's see if this causes an unhandled panic (e.g. overflow not gracefully caught).
        // parse_greek_numeral uses `checked_mul` so it shouldn't panic, but Havoc tries anyway!
        let _ = parse_greek_numeral(&numeral);
    }
}
