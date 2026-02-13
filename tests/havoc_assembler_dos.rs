use glossa::ast::Expr;
use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::Assembler;

#[test]
fn test_assembler_adjective_dos() {
    let mut asm = Assembler::new();
    let adj = MorphAnalysis {
        lemma: "καλός".into(),
        part_of_speech: PartOfSpeech::Adjective,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Feed 2000 adjectives (limit should be 1024)
    let mut failed = false;
    for _ in 0..2000 {
        if asm.feed(&adj, "καλός").is_err() {
            failed = true;
            break;
        }
    }

    // The test expects the assembler to REJECT the excessive input
    assert!(
        failed,
        "Assembler accepted infinite adjectives (DoS vulnerability)"
    );
}

#[test]
fn test_assembler_literal_dos() {
    let mut asm = Assembler::new();

    // Feed 2000 strings (limit should be 1024)
    let mut failed = false;
    for i in 0..2000 {
        if asm.feed_string(format!("test_{}", i)).is_err() {
            failed = true;
            break;
        }
    }

    assert!(
        failed,
        "Assembler accepted infinite string literals (DoS vulnerability)"
    );
}

#[test]
fn test_assembler_array_dos() {
    let mut asm = Assembler::new();
    let elements = vec![Expr::NumberLiteral(1)];

    // Feed 1000 arrays (limit should be 256)
    let mut failed = false;
    for _ in 0..1000 {
        if asm.feed_array(elements.clone()).is_err() {
            failed = true;
            break;
        }
    }

    assert!(
        failed,
        "Assembler accepted infinite arrays (DoS vulnerability)"
    );
}

#[test]
fn test_assembler_nested_phrase_dos() {
    let mut asm = Assembler::new();
    let elements = vec![Expr::NumberLiteral(1)];

    // Feed 1000 nested phrases (limit should be 256)
    let mut failed = false;
    for _ in 0..1000 {
        if asm.feed_nested_phrase(elements.clone()).is_err() {
            failed = true;
            break;
        }
    }

    assert!(
        failed,
        "Assembler accepted infinite nested phrases (DoS vulnerability)"
    );
}
