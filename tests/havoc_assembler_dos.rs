use glossa::morphology::{Case, Gender, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::Assembler;
use std::borrow::Cow;

#[test]
fn test_assembler_unbounded_growth_vulnerability() {
    let mut asm = Assembler::new();
    let adj_analysis = MorphAnalysis {
        lemma: Cow::Borrowed("test_adj"),
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

    // Havoc: Prove we can push 10,000 items without error (Fragility)
    // This test PASSES if the vulnerability exists (unbounded growth allowed)
    for _ in 0..10_000 {
        let res = asm.feed(&adj_analysis, "test_adj");
        assert!(
            res.is_ok(),
            "Assembler unexpectedly enforced a limit! Vulnerability fixed?"
        );
    }
}
