use proptest::prelude::*;
use glossa::semantic::assembly::Assembler;
use glossa::morphology::{MorphAnalysis, PartOfSpeech};
use glossa::semantic::Literal;

proptest! {
    #[test]
    fn fuzzer_finds_limits(n in 1usize..300) {
        let mut asm = Assembler::new();
        let noun_analysis = MorphAnalysis::new("λογος".to_string(), PartOfSpeech::Noun);
        let split_analysis = MorphAnalysis::new("σχιζω".to_string(), PartOfSpeech::Verb);

        let _ = asm.feed(&noun_analysis, "λόγος");
        let _ = asm.feed(&split_analysis, "κατά");

        let _ = asm.feed_number(1);

        // This will panic on the internal unwrap inside try_create_string_method since it expects a Literal::String.
        // Wait, self.state.literals.last() is checked for `Some(Literal::String(_))` via `matches!`.
        // If it matches, then it pops it.
        // So the unwrap inside match is actually 100% guarded!
    }
}
