#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;
use glossa::morphology::{Case, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::Assembler;
use std::borrow::Cow;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        if s.len() > 1024 { return; } // prevent slow fuzzing
        let mut asm = Assembler::new();
        let analysis = MorphAnalysis {
            lemma: Cow::Owned(s.to_lowercase()),
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
        let _ = asm.feed(&analysis, &s);
        let _ = asm.finalize();
    }
});
