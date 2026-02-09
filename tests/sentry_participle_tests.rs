use glossa::morphology::{Case, Gender, Number, Tense, Voice, participle::analyze_participle};

#[test]
fn test_participle_analysis_coverage() {
    struct TestCase {
        word: &'static str,
        expected_stem: &'static str,
        expected_tense: Tense,
        expected_voice: Voice,
        expected_case: Case,
        expected_gender: Gender,
        expected_number: Number,
    }

    let cases = vec![
        // Present Active (Singular)
        TestCase {
            word: "γραφων",
            expected_stem: "γραφ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Active,
            expected_case: Case::Nominative,
            expected_gender: Gender::Masculine,
            expected_number: Number::Singular,
        },
        TestCase {
            word: "γραφουσα",
            expected_stem: "γραφ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Active,
            expected_case: Case::Nominative,
            expected_gender: Gender::Feminine,
            expected_number: Number::Singular,
        },
        TestCase {
            word: "γραφον",
            expected_stem: "γραφ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Active,
            expected_case: Case::Nominative, // or Accusative (ambiguous, but checks first match)
            expected_gender: Gender::Neuter,
            expected_number: Number::Singular,
        },
        // Present Active (Plural)
        TestCase {
            word: "γραφοντες",
            expected_stem: "γραφ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Active,
            expected_case: Case::Nominative,
            expected_gender: Gender::Masculine,
            expected_number: Number::Plural,
        },
        // Present Middle/Passive
        TestCase {
            word: "λυομενος",
            expected_stem: "λυ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Middle,
            expected_case: Case::Nominative,
            expected_gender: Gender::Masculine,
            expected_number: Number::Singular,
        },
        TestCase {
            word: "λυομενη",
            expected_stem: "λυ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Middle,
            expected_case: Case::Nominative,
            expected_gender: Gender::Feminine,
            expected_number: Number::Singular,
        },
        TestCase {
            word: "λυομενον",
            expected_stem: "λυ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Middle,
            expected_case: Case::Nominative, // or Accusative, Neuter
            expected_gender: Gender::Neuter, // or Masculine Accusative
            expected_number: Number::Singular,
        },
        // Overlapping suffixes check: λυομενον ends in "ον"
        // If sorting is broken, it might match "ον" (Present Active) -> stem "λυομεν"
        // Correct behavior: matches "ομενον" (Present Middle) -> stem "λυ"
        TestCase {
            word: "λυομενον",
            expected_stem: "λυ",
            expected_tense: Tense::Present,
            expected_voice: Voice::Middle,
            expected_case: Case::Nominative,
            expected_gender: Gender::Neuter,
            expected_number: Number::Singular,
        },
        // Aorist Active
        TestCase {
            word: "λυσας", // stem λυσ -> lemma λυσω (not strictly true lemma which is λυω, but analyze_participle uses stem+ω)
            // Actually, analyze_participle returns stem as-is.
            // For aorist "λυσας", stem is "λυσ". verb_lemma() -> "λυσω".
            // The existing code doesn't strip sigmatic aorist suffix "σ" from participle stem.
            // Let's check what the code does.
            // analyze_participle("γραψας") -> stem "γραψ"
            expected_stem: "λυσ",
            expected_tense: Tense::Aorist,
            expected_voice: Voice::Active,
            expected_case: Case::Nominative,
            expected_gender: Gender::Masculine,
            expected_number: Number::Singular,
        },
        TestCase {
            word: "λυσαντος",
            expected_stem: "λυσ",
            expected_tense: Tense::Aorist,
            expected_voice: Voice::Active,
            expected_case: Case::Genitive,
            expected_gender: Gender::Masculine,
            expected_number: Number::Singular,
        },
        // Perfect Passive
        TestCase {
            word: "λελυμενος",
            expected_stem: "λελυ",
            expected_tense: Tense::Perfect,
            expected_voice: Voice::Passive,
            expected_case: Case::Nominative,
            expected_gender: Gender::Masculine,
            expected_number: Number::Singular,
        },
    ];

    for (i, test) in cases.iter().enumerate() {
        let result = analyze_participle(test.word);
        assert!(
            result.is_some(),
            "TestCase #{}: Failed to analyze participle '{}'",
            i,
            test.word
        );

        let analysis = result.unwrap();
        assert_eq!(
            analysis.stem, test.expected_stem,
            "TestCase #{}: Incorrect stem for '{}'",
            i, test.word
        );
        assert_eq!(
            analysis.tense, test.expected_tense,
            "TestCase #{}: Incorrect tense for '{}'",
            i, test.word
        );
        assert_eq!(
            analysis.voice, test.expected_voice,
            "TestCase #{}: Incorrect voice for '{}'",
            i, test.word
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
            analysis.case, test.expected_case,
            "TestCase #{}: Incorrect case for '{}'",
            i, test.word
        );
        assert_eq!(
            analysis.gender, test.expected_gender,
            "TestCase #{}: Incorrect gender for '{}'",
            i, test.word
        );
        assert_eq!(
            analysis.number, test.expected_number,
            "TestCase #{}: Incorrect number for '{}'",
            i, test.word
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
