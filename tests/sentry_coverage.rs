use glossa::morphology::{Case, analyze};
use glossa::semantic::Assembler;

#[test]
fn should_return_error_when_double_indirect() {
    let mut asm = Assembler::new();
    let analysis1 = analyze("ἀνθρώπῳ"); // Dative
    let analysis2 = analyze("θεῷ"); // Dative

    assert!(analysis1.case == Some(Case::Dative));
    assert!(analysis2.case == Some(Case::Dative));

    asm.feed(&analysis1, "ἀνθρώπῳ").unwrap();
    let result = asm.feed(&analysis2, "θεῷ");

    match result {
        Err(glossa::errors::AssemblyError::DoubleIndirect) => {}
        _ => panic!("Expected DoubleIndirect, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_double_object() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis1 = analyze("λόγον"); // Accusative
    let analysis2 = analyze("κόσμον"); // Accusative

    assert!(analysis1.case == Some(Case::Accusative));
    assert!(analysis2.case == Some(Case::Accusative));

    asm.feed(&analysis1, "λόγον").unwrap();
    let result = asm.feed(&analysis2, "κόσμον");

    match result {
        Err(glossa::errors::AssemblyError::DoubleObject) => {}
        _ => panic!("Expected DoubleObject, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_double_verb() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis1 = analyze("λέγει"); // Verb
    let analysis2 = analyze("γράφει"); // Verb

    asm.feed(&analysis1, "λέγει").unwrap();
    let result = asm.feed(&analysis2, "γράφει");

    match result {
        Err(glossa::errors::AssemblyError::DoubleVerb) => {}
        _ => panic!("Expected DoubleVerb, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_subject_verb_disagreement_plural_verb() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj = analyze("ἄνθρωπος"); // Singular subject
    let analysis_verb = analyze("λέγουσιν"); // Plural verb

    asm.feed(&analysis_subj, "ἄνθρωπος").unwrap();
    let result = asm.feed(&analysis_verb, "λέγουσιν");
    match result {
        Err(glossa::errors::AssemblyError::SubjectVerbDisagreement { subject, verb }) => {
            assert_eq!(subject.1, Some(glossa::morphology::Number::Singular));
            assert_eq!(verb.1, Some(glossa::morphology::Number::Plural));
        }
        _ => panic!("Expected SubjectVerbDisagreement, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_subject_verb_disagreement_person() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj = analyze("ἐγώ"); // 1st person
    let analysis_verb = analyze("λέγει"); // 3rd person

    asm.feed(&analysis_subj, "ἐγώ").unwrap();
    let result = asm.feed(&analysis_verb, "λέγει");

    match result {
        Err(glossa::errors::AssemblyError::SubjectVerbDisagreement { subject, verb }) => {
            assert_eq!(subject.0, Some(glossa::morphology::Person::First));
            assert_eq!(verb.0, Some(glossa::morphology::Person::Third));
        }
        _ => panic!("Expected SubjectVerbDisagreement, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_subject_verb_disagreement_reverse() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj = analyze("ἄνθρωπος"); // 3rd person
    let analysis_verb = analyze("λέγουσιν"); // 3rd person, Plural

    // Feed verb first
    asm.feed(&analysis_verb, "λέγουσιν").unwrap();
    let result = asm.feed(&analysis_subj, "ἄνθρωπος");

    match result {
        Err(glossa::errors::AssemblyError::SubjectVerbDisagreement { subject, verb }) => {
            assert_eq!(subject.1, Some(glossa::morphology::Number::Singular));
            assert_eq!(verb.1, Some(glossa::morphology::Number::Plural));
        }
        _ => panic!("Expected SubjectVerbDisagreement, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_missing_verb() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj = analyze("ἄνθρωπος");
    let analysis_obj = analyze("λόγον");

    asm.feed(&analysis_subj, "ἄνθρωπος").unwrap();
    asm.feed(&analysis_obj, "λόγον").unwrap();

    let result = asm.finalize();
    match result {
        Err(glossa::errors::AssemblyError::MissingVerb) => {}
        _ => panic!("Expected MissingVerb, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_double_subject() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj1 = analyze("ἄνθρωπος");
    let analysis_subj2 = analyze("θεός");

    asm.feed(&analysis_subj1, "ἄνθρωπος").unwrap();
    asm.feed(&analysis_subj2, "θεός").unwrap();
    let result = asm.finalize();
    match result {
        Err(glossa::errors::AssemblyError::DoubleSubject) => {}
        _ => panic!("Expected DoubleSubject, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_subject_verb_disagreement_neuter_plural() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj = analyze("ζῷα"); // Neuter plural
    let analysis_verb = analyze("λέγει"); // Singular verb

    asm.feed(&analysis_subj, "ζῷα").unwrap();
    asm.feed(&analysis_verb, "λέγει").unwrap();

    let result = asm.finalize();
    // Neuter plural subject with singular verb should be perfectly valid in Ancient Greek!
    assert!(result.is_ok());
}

#[test]
fn should_return_error_when_subject_verb_disagreement_neuter_plural_invalid() {
    let mut asm = glossa::semantic::Assembler::new();
    let analysis_subj = analyze("ζῷα"); // Neuter plural
    let analysis_verb = analyze("λέγουσιν"); // Plural verb

    asm.feed(&analysis_subj, "ζῷα").unwrap();
    let result = asm.feed(&analysis_verb, "λέγουσιν");

    // Neuter plural subject with plural verb should actually be invalid because Ancient Greek rule is neuter plural -> singular verb!
    match result {
        Err(glossa::errors::AssemblyError::SubjectVerbDisagreement { subject, verb }) => {
            // Since neuter plural is treated as singular for agreement
            assert_eq!(subject.1, Some(glossa::morphology::Number::Singular));
            assert_eq!(verb.1, Some(glossa::morphology::Number::Plural));
        }
        _ => panic!("Expected SubjectVerbDisagreement, got {:?}", result),
    }
}

#[test]
fn should_return_error_when_limit_exceeded_arrays() {
    let mut asm = glossa::semantic::Assembler::new();

    // Max Arrays is 256. We feed 257.
    for _ in 0..256 {
        asm.feed_array(vec![]).unwrap();
    }

    let result = asm.feed_array(vec![]);

    match result {
        Err(glossa::errors::AssemblyError::LimitExceeded { resource, max }) => {
            assert_eq!(resource, "Arrays");
            assert_eq!(max, 256);
        }
        _ => panic!("Expected LimitExceeded, got {:?}", result),
    }
}
