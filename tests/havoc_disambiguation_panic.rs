use glossa::morphology::{Case, DisambiguationContext, MorphAnalysis, PartOfSpeech, disambiguate};

#[test]
// #[should_panic] // No longer should panic
fn test_disambiguate_nan_panic() {
    let nan_analysis =
        MorphAnalysis::new("test".to_string(), PartOfSpeech::Noun).with_confidence(f32::NAN);

    let valid_analysis =
        MorphAnalysis::new("valid".to_string(), PartOfSpeech::Noun).with_confidence(1.0);

    let analyses = vec![nan_analysis, valid_analysis];

    // Must provide context to bypass the "no context" early return
    let mut context = DisambiguationContext::new();
    context.expected_case = Some(Case::Nominative);

    // This should NOT panic now
    disambiguate(analyses, &context);
}
