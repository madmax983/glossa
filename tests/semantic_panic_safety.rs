use glossa::morphology::{Case, MorphAnalysis, PartOfSpeech};
use glossa::semantic::{DisambiguationContext, disambiguate};

#[test]
fn test_disambiguate_handles_nan_confidence_safely() {
    let analyses = vec![
        MorphAnalysis::new("test1".to_string(), PartOfSpeech::Noun).with_confidence(1.0),
        MorphAnalysis::new("test2".to_string(), PartOfSpeech::Noun).with_confidence(f32::NAN),
    ];

    // Create a context that will trigger scoring
    let context = DisambiguationContext::expecting_case(Case::Nominative);

    // This should NOT panic now
    let result = disambiguate(analyses, &context);

    // Check that we got results back
    assert_eq!(result.len(), 2);
    // The order is undefined for NaN, but it shouldn't crash
}
