#![allow(missing_docs)]
use glossa::morphology::{Case, Gender, Number, Tense, Voice, analyze_participle};

#[test]
fn test_participle_analysis_coverage() {
    let cases = vec![
        // Present Active (Singular)
        (
            "γραφων",
            "γραφ",
            Tense::Present,
            Voice::Active,
            Case::Nominative,
            Gender::Masculine,
            Number::Singular,
        ),
        (
            "γραφουσα",
            "γραφ",
            Tense::Present,
            Voice::Active,
            Case::Nominative,
            Gender::Feminine,
            Number::Singular,
        ),
        (
            "γραφον",
            "γραφ",
            Tense::Present,
            Voice::Active,
            Case::Nominative, // or Accusative (ambiguous, but checks first match)
            Gender::Neuter,
            Number::Singular,
        ),
        // Present Active (Plural)
        (
            "γραφοντες",
            "γραφ",
            Tense::Present,
            Voice::Active,
            Case::Nominative,
            Gender::Masculine,
            Number::Plural,
        ),
        // Present Middle/Passive
        (
            "λυομενος",
            "λυ",
            Tense::Present,
            Voice::Middle,
            Case::Nominative,
            Gender::Masculine,
            Number::Singular,
        ),
        (
            "λυομενη",
            "λυ",
            Tense::Present,
            Voice::Middle,
            Case::Nominative,
            Gender::Feminine,
            Number::Singular,
        ),
        (
            "λυομενον",
            "λυ",
            Tense::Present,
            Voice::Middle,
            Case::Nominative, // or Accusative, Neuter
            Gender::Neuter,   // or Masculine Accusative
            Number::Singular,
        ),
        // Overlapping suffixes check: λυομενον ends in "ον"
        // If sorting is broken, it might match "ον" (Present Active) -> stem "λυομεν"
        // Correct behavior: matches "ομενον" (Present Middle) -> stem "λυ"
        (
            "λυομενον",
            "λυ",
            Tense::Present,
            Voice::Middle,
            Case::Nominative,
            Gender::Neuter,
            Number::Singular,
        ),
        // Aorist Active
        (
            "λυσας", // stem λυσ -> lemma λυσω (not strictly true lemma which is λυω, but analyze_participle uses stem+ω)
            // Actually, analyze_participle returns stem as-is.
            // For aorist "λυσας", stem is "λυσ". verb_lemma() -> "λυσω".
            // The existing code doesn't strip sigmatic aorist suffix "σ" from participle stem.
            // Let's check what the code does.
            // analyze_participle("γραψας") -> stem "γραψ"
            "λυσ",
            Tense::Aorist,
            Voice::Active,
            Case::Nominative,
            Gender::Masculine,
            Number::Singular,
        ),
        (
            "λυσαντος",
            "λυσ",
            Tense::Aorist,
            Voice::Active,
            Case::Genitive,
            Gender::Masculine,
            Number::Singular,
        ),
        // Perfect Passive
        (
            "λελυμενος",
            "λελυ",
            Tense::Perfect,
            Voice::Passive,
            Case::Nominative,
            Gender::Masculine,
            Number::Singular,
        ),
    ];

    for (
        i,
        (
            word,
            expected_stem,
            expected_tense,
            expected_voice,
            expected_case,
            expected_gender,
            expected_number,
        ),
    ) in cases.into_iter().enumerate()
    {
        let result = analyze_participle(word);
        assert!(
            result.is_some(),
            "TestCase #{}: Failed to analyze participle '{}'",
            i,
            word
        );

        let analysis = result.unwrap();
        assert_eq!(
            analysis.stem, expected_stem,
            "TestCase #{}: Incorrect stem for '{}'",
            i, word
        );
        assert_eq!(
            analysis.tense, expected_tense,
            "TestCase #{}: Incorrect tense for '{}'",
            i, word
        );
        assert_eq!(
            analysis.voice, expected_voice,
            "TestCase #{}: Incorrect voice for '{}'",
            i, word
        );
        // Note: For ambiguous cases (like neuter nom/acc), we check what the parser returns first.
        // The implementation sorts patterns by length, but identical length order depends on declaration order.
        // In `participle.rs`:
        // Neuter singular (-ον, -οντος, -οντι)
        // ParticiplePattern { ending: "ον", case: Nominative ... } comes before Accusative.
        // So we expect Nominative.
        // For `λυομενον`:
        // Neuter singular (-ομενον) Nominative comes before Accusative.
        // And Neuter comes before Masculine Accusative.
        // So expected values in test cases above (Nominative, Neuter) are correct based on source order.
        assert_eq!(
            analysis.case, expected_case,
            "TestCase #{}: Incorrect case for '{}'",
            i, word
        );
        assert_eq!(
            analysis.gender, expected_gender,
            "TestCase #{}: Incorrect gender for '{}'",
            i, word
        );
        assert_eq!(
            analysis.number, expected_number,
            "TestCase #{}: Incorrect number for '{}'",
            i, word
        );
    }
}

#[test]
fn test_participle_lemma_generation() {
    // Lemma generation is just format!("{}ω", stem)
    // But it's good to verify behavior for stems ending in vowels/consonants

    // Consonant stem
    let p = analyze_participle("γραφων").unwrap();
    assert_eq!(p.verb_lemma(), "γραφω");

    // Vowel stem
    // e.g. ποιέω -> ποιῶν (contracted). If we input uncontracted "ποιεων" (artificial)
    // "ποιεων" -> stem "ποιε" -> lemma "ποιεω"
    if let Some(p) = analyze_participle("ποιεων") {
        assert_eq!(p.verb_lemma(), "ποιεω");
    }

    // Aorist stem
    // "λυσας" -> stem "λυσ" -> lemma "λυσω" (Future-like or Aorist Subjunctive-like form)
    // Note: The true dictionary lemma is "λυω". The current implementation of `verb_lemma`
    // just appends 'ω' to the stem found. It does NOT strip aorist markers (sigma).
    // This is documented behavior ("The verb stem (without participle suffix)").
    let p = analyze_participle("λυσας").unwrap();
    assert_eq!(p.verb_lemma(), "λυσω");
}

#[test]
fn test_invalid_participles() {
    // Random words that shouldn't match
    assert!(analyze_participle("λογος").is_none());

    // Words that end in "ον" but stem is empty?
    // "ον" -> stem "". Code says: if stem.is_empty() { continue; }
    assert!(analyze_participle("ον").is_none());

    // "ων" -> stem "".
    assert!(analyze_participle("ων").is_none());
}
