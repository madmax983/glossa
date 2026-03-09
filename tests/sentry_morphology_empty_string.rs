use glossa::morphology::analyze_noun;
use glossa::morphology::analyze_participle;
use glossa::morphology::analyze_verb;

#[test]
fn test_morphology_empty_string_does_not_panic() {
    assert!(analyze_noun("").is_none());
    assert!(analyze_verb("").is_none());
    assert!(analyze_participle("").is_none());
}

#[test]
fn test_morphology_invalid_unicode() {
    assert!(analyze_noun("invalid\u{FFFD}chars").is_none());
    assert!(analyze_verb("invalid\u{FFFD}chars").is_none());
    assert!(analyze_participle("invalid\u{FFFD}chars").is_none());
}
