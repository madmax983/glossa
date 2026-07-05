#![allow(missing_docs)]
use glossa::morphology::{Case, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::assembly::Assembler;
use proptest as prop;
use proptest::prelude::*;
use std::borrow::Cow;

proptest! {
    #[test]
    fn test_assembler_no_panic(
        words in prop::collection::vec("[a-zA-Z0-9]+", 0..10)
    ) {
        let mut asm = Assembler::new();
        for w in words {
            // Fake some MorphAnalysis
            let analysis = MorphAnalysis {
                lemma: Cow::Owned(w.to_lowercase()),
                part_of_speech: PartOfSpeech::Noun,
                case: Some(Case::Nominative),
                number: Some(Number::Singular),
                gender: None,
                person: None,
                tense: None,
                mood: None,
                voice: None,
                confidence: 1.0,
            };
            let _ = asm.feed(&analysis, &w);
        }
        let _ = asm.finalize();
    }
}

proptest! {
    #[test]
    fn test_assembler_full_panic(
        words in prop::collection::vec(
            (
                "[a-zA-Z0-9]+", // original
                "[a-zA-Z0-9]+", // lemma
                prop::sample::select(vec![PartOfSpeech::Noun, PartOfSpeech::Verb, PartOfSpeech::Adjective, PartOfSpeech::Pronoun, PartOfSpeech::Article, PartOfSpeech::Conjunction, PartOfSpeech::Preposition, PartOfSpeech::Numeral, PartOfSpeech::Particle, PartOfSpeech::Unknown]),
                prop::option::of(prop::sample::select(vec![Case::Nominative, Case::Genitive, Case::Dative, Case::Accusative, Case::Vocative])),
                prop::option::of(prop::sample::select(vec![Number::Singular, Number::Plural])),
            ), 0..20
        )
    ) {
        let mut asm = Assembler::new();
        for (w, l, pos, case, number) in words {
            let analysis = MorphAnalysis {
                lemma: Cow::Owned(l),
                part_of_speech: pos,
                case,
                number,
                gender: None,
                person: None,
                tense: None,
                mood: None,
                voice: None,
                confidence: 1.0,
            };
            let _ = asm.feed(&analysis, &w);
        }
        let _ = asm.finalize();
    }
}
