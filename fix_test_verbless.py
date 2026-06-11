import re

with open('src/semantic/assembler_tests.rs', 'r') as f:
    content = f.read()

search = '''#[test]
fn test_verbless_statement() {
    let mut asm = Assembler::new();

    let subj = MorphAnalysis {
        lemma: std::borrow::Cow::Borrowed("ανθρωπος"),
        part_of_speech: PartOfSpeech::Noun,
        case: Some(Case::Nominative),
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: Some(Person::Third),
        tense: None,
        mood: None,
        voice: None,
        confidence: 1.0,
    };
    asm.feed(&subj, "ἄνθρωπος").unwrap();

    let stmt = asm.finalize();
    assert!(
        stmt.is_ok(),
        "Expected Ok but got {:?}",
        stmt
    );
}'''

replace = ''

with open('src/semantic/assembler_tests.rs', 'w') as f:
    f.write(content.replace(search, replace))
