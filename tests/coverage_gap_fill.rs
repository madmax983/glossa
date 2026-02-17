use glossa::morphology::{Case, MorphAnalysis, Number, PartOfSpeech};
use glossa::semantic::Assembler;
use std::borrow::Cow;

#[test]
fn test_conjunction_passthrough() {
    let mut asm = Assembler::new();

    // "ὅτι" (that) is a conjunction but not an operator
    let conj = MorphAnalysis {
        lemma: Cow::Borrowed("οτι"),
        part_of_speech: PartOfSpeech::Conjunction,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // Should return Ok(()) and not affect state (except maybe consumption if we tracked it)
    asm.feed(&conj, "ὅτι").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(stmt.operators.is_empty());
    assert!(stmt.subject.is_none());
}

#[test]
fn test_numeral_pos_passthrough() {
    let mut asm = Assembler::new();

    // "πέντε" (five) as a Numeral POS (not just a literal check)
    // feed_with_normalized checks special_properties (numeral_value) first.
    // If we want to hit the PartOfSpeech::Numeral branch, we need a word that
    // is POS::Numeral but NOT caught by lexicon::numeral_value?
    // OR, we assume lexicon::numeral_value catches it and returns Ok(true),
    // causing early return.
    // Wait, `check_special_properties` returns `Ok(true)` if found.
    // So `match analysis.part_of_speech` is NOT reached for standard numerals.
    // To hit `PartOfSpeech::Numeral` branch, we need a word that is analyzed as Numeral
    // but `numeral_value` returns None.
    // That implies a lexicon definition mismatch or a "variable" numeral?
    // Or maybe I should just check if `check_special_properties` handles it.
    // Actually, checking `src/semantic/assembler.rs`:
    // `if self.check_special_properties(normalized)? { return Ok(()); }`
    // `check_special_properties` calls `lexicon::numeral_value`.
    // If that returns Some, it pushes literal and returns true.
    // So the `PartOfSpeech::Numeral` branch in `match` is DEAD CODE unless
    // there is a numeral that isn't in `numeral_value` map but is analyzed as Numeral.
    // `MorphAnalysis` comes from `analyze`. `analyze` uses lexicon/numerals.
    // If I manually construct an analysis...

    let fake_numeral = MorphAnalysis {
        lemma: Cow::Borrowed("unknown_numeral"),
        part_of_speech: PartOfSpeech::Numeral,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    // feed this. `check_special_properties` will look up "unknown_numeral" in `numeral_value`.
    // It should return None.
    // Then it proceeds to `match`.
    // Hits `PartOfSpeech::Numeral` => `handle_nominal`.
    // Should populate subject.

    asm.feed(&fake_numeral, "unknown_numeral").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(stmt.subject.is_some());
    assert_eq!(stmt.subject.unwrap().lemma, "unknown_numeral");
}

#[test]
fn test_unknown_pos_ignored() {
    let mut asm = Assembler::new();

    // Feed a Particle (not handled in match)
    let particle = MorphAnalysis {
        lemma: Cow::Borrowed("particle"),
        part_of_speech: PartOfSpeech::Particle,
        case: None,
        number: None,
        gender: None,
        person: None,
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };

    asm.feed(&particle, "particle").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(stmt.subject.is_none());
}
