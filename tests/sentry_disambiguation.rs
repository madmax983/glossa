use glossa::morphology::{
    disambiguate, resolve_best, Case, DisambiguationContext, Gender, MorphAnalysis, Number,
    PartOfSpeech, Person,
};

#[test]
fn test_should_set_expected_case_gender_number_from_article() {
    let mut article = MorphAnalysis::new("ο".to_string(), PartOfSpeech::Article);
    article.case = Some(Case::Nominative);
    article.gender = Some(Gender::Masculine);
    article.number = Some(Number::Singular);
    let ctx = DisambiguationContext::from_article(&article);
    assert_eq!(ctx.expected_case, Some(Case::Nominative));
    assert_eq!(ctx.expected_gender, Some(Gender::Masculine));
    assert_eq!(ctx.expected_number, Some(Number::Singular));
    assert_eq!(ctx.expected_person, None);
}

#[test]
fn test_should_set_expected_nominative_number_person_from_verb() {
    let mut verb = MorphAnalysis::new("λεγω".to_string(), PartOfSpeech::Verb);
    verb.number = Some(Number::Singular);
    verb.person = Some(Person::First);
    let ctx = DisambiguationContext::from_verb(&verb);
    assert_eq!(ctx.expected_case, Some(Case::Nominative));
    assert_eq!(ctx.expected_number, Some(Number::Singular));
    assert_eq!(ctx.expected_person, Some(Person::First));
    assert_eq!(ctx.expected_gender, None);
}

#[test]
fn test_should_set_expected_case_only() {
    let ctx = DisambiguationContext::expecting_case(Case::Dative);
    assert_eq!(ctx.expected_case, Some(Case::Dative));
    assert_eq!(ctx.expected_number, None);
    assert_eq!(ctx.expected_gender, None);
    assert_eq!(ctx.expected_person, None);
}

#[test]
fn test_should_return_empty_when_disambiguating_empty_analyses() {
    let analyses = vec![];
    let ctx = DisambiguationContext::expecting_case(Case::Dative);
    let resolved = disambiguate(analyses, &ctx);
    assert!(resolved.is_empty());
}

#[test]
fn test_should_penalize_confidence_on_context_mismatch() {
    let mut analysis = MorphAnalysis::new("λογου".to_string(), PartOfSpeech::Noun);
    analysis.case = Some(Case::Genitive);
    analysis.number = Some(Number::Singular);
    analysis.gender = Some(Gender::Masculine);
    analysis.person = Some(Person::Third);

    let mut ctx = DisambiguationContext::new();
    ctx.expected_case = Some(Case::Nominative);
    ctx.expected_number = Some(Number::Plural);
    ctx.expected_gender = Some(Gender::Feminine);
    ctx.expected_person = Some(Person::First);

    // This tests lines 171, 180, 189, 198 (mismatch penalties)
    let analyses = vec![analysis];
    let resolved = disambiguate(analyses, &ctx);

    // After penalties, confidence should be lower (capped at 0.0)
    assert!(resolved[0].confidence < 1.0);
}

#[test]
fn test_should_return_unknown_morph_analysis_when_resolving_best_empty() {
    let analyses = vec![];
    let ctx = DisambiguationContext::expecting_case(Case::Dative);
    let best = resolve_best(analyses, &ctx);
    assert_eq!(best.lemma, "?");
    assert_eq!(best.part_of_speech, PartOfSpeech::Unknown);
}
