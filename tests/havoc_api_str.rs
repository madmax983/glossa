use proptest::prelude::*;

proptest! {
    #[test]
    fn test_text_normalize_does_not_crash(s in "\\PC*") {
        glossa::text::normalize_greek(&s);
    }

    #[test]
    fn test_morphology_analyze_verb_does_not_crash(s in "\\PC*") {
        // Need to run via analyze_program to exercise full parser and morphological integration
        // since analyze_verb is private
        let _ = glossa::tools::runner::analyze_source(&s);
    }
}
