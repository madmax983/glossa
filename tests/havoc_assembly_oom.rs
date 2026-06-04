use glossa::morphology::{MorphAnalysis, PartOfSpeech};
use glossa::semantic::assembly::Assembler;
use std::borrow::Cow;

#[test]
#[ignore]
fn test_assembler_memory_exhaustion() {
    let mut asm = Assembler::new();
    let analysis = MorphAnalysis {
        part_of_speech: PartOfSpeech::Noun,
        lemma: Cow::Owned("test".to_string()),
        person: None,
        number: None,
        tense: None,
        mood: None,
        voice: None,
        gender: None,
        case: None,
        confidence: 1.0,
    };

    for _ in 0..10_000 {
        let _ = asm.feed(&analysis, "εἷς");
    }
}
