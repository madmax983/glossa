#[cfg(test)]
mod tests {
    use glossa::morphology::{Mood, Tense, Voice, analyze_verb};

    #[test]
    fn test_trim_end_matches_bug_reproduction() {
        // We want to simulate a case where the stem ends in 'θ' (or multiple 'θ's)
        // and check if `trim_end_matches` over-strips.
        // The pattern AORIST_PASSIVE_OPT has endings like "θειη".
        // If we provide a word that matches this ending, and the remaining stem ends in "θ",
        // the current logic strips *all* trailing thetas.

        // Construct a word: "αθθ" (stem) + "θειη" (ending) -> "αθθθειη"
        // (Note: This is not valid Greek phonology, but we are testing string handling logic)
        let word = "αθθθειη";

        // This should match AORIST_PASSIVE_OPT
        let analysis = analyze_verb(word).expect("Should analyze as verb");

        assert_eq!(analysis.tense, Some(Tense::Aorist));
        assert_eq!(analysis.mood, Some(Mood::Optative));
        assert_eq!(analysis.voice, Some(Voice::Passive));

        // Current buggy behavior: trim_end_matches('θ') removes ALL thetas from "αθθ" -> "α"
        // Lemma becomes "αω"
        // Correct behavior (if we must strip one): strip_suffix('θ') -> "αθ"
        // Lemma becomes "αθω"

        // We assert the BUGGY behavior first to prove reproduction (Red Phase)
        // Or better, we assert the CORRECT behavior and expect it to fail.
        // Warden philosophy: Red Phase = Write tests that fail.

        assert_eq!(
            analysis.lemma, "αθω",
            "Logic bug: trim_end_matches over-stripped the stem 'αθθ'"
        );
    }
}
