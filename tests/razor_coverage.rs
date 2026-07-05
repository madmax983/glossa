#![allow(missing_docs)]
//! Coverage tests for refactored modules (errors.rs and merged assembler.rs)
//!
//! This file aims to exercise the code paths that were moved during refactoring
//! to ensure high code coverage.

use glossa::errors::{AssemblyError, GlossaError};
use glossa::morphology::{Case, Gender, Number, Person, Tense, Voice};
use glossa::semantic::{
    AssembledStatement, Constituent, Literal, ParticipleConstituent, VerbConstituent,
};
use miette::Diagnostic;
use smol_str::SmolStr;

// --- Errors Coverage ---

#[test]
fn test_errors_display_impls() {
    // Test GlossaError variants
    let err_parse = GlossaError::parse("test");
    assert!(format!("{}", err_parse).contains("Σφάλμα συντάξεως"));
    assert_eq!(err_parse.category_greek(), "Σύνταξις");

    let err_semantic = GlossaError::semantic("test");
    assert!(format!("{}", err_semantic).contains("Σφάλμα σημασίας"));
    assert_eq!(err_semantic.category_greek(), "Σημασία");

    let err_undefined = GlossaError::undefined("x");
    assert!(format!("{}", err_undefined).contains("Ἄγνωστον ὄνομα"));
    assert_eq!(err_undefined.category_greek(), "Ὄνομα");

    let err_agreement = GlossaError::agreement("test");
    assert!(format!("{}", err_agreement).contains("Σφάλμα συμφωνίας"));
    assert_eq!(err_agreement.category_greek(), "Συμφωνία");

    let err_codegen = GlossaError::codegen("test");
    assert!(format!("{}", err_codegen).contains("Σφάλμα κώδικος"));
    assert_eq!(err_codegen.category_greek(), "Κῶδιξ");

    let err_limit = GlossaError::LimitExceeded {
        resource: "test".into(),
        max: 10,
    };
    assert!(format!("{}", err_limit).contains("Ὑπέρβασις ὀρίου"));
    assert_eq!(err_limit.category_greek(), "Όριον");

    let err_assembly: GlossaError = AssemblyError::DoubleSubject.into();
    assert!(format!("{}", err_assembly).contains("Διπλοῦν ὑποκείμενον"));
    assert_eq!(err_assembly.category_greek(), "Συναρμογή");
}

#[test]
fn test_assembly_errors_display_impls() {
    let err = AssemblyError::DoubleSubject;
    assert!(format!("{}", err).contains("Διπλοῦν ὑποκείμενον"));
    assert!(err.code().unwrap().to_string().contains("double_subject"));

    let err = AssemblyError::DoubleObject;
    assert!(format!("{}", err).contains("Διπλοῦν ἀντικείμενον"));

    let err = AssemblyError::DoubleIndirect;
    assert!(format!("{}", err).contains("Διπλοῦν ἔμμεσον αντικείμενον"));

    let err = AssemblyError::DoubleVerb;
    assert!(format!("{}", err).contains("Διπλοῦν ῥῆμα"));

    let err = AssemblyError::SubjectVerbDisagreement {
        subject: (Some(Person::First), Some(Number::Singular)),
        verb: (Some(Person::Third), Some(Number::Singular)),
    };
    assert!(format!("{}", err).contains("Ἀσυμφωνία"));

    let err = AssemblyError::LimitExceeded {
        resource: "x".into(),
        max: 5,
    };
    assert!(format!("{}", err).contains("Ὑπέρβασις ὁρίου"));
}

// --- Assembler Structs Coverage ---

#[test]
fn test_assembler_structs_derive_coverage() {
    // VerbConstituent
    let v = VerbConstituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        person: Some(Person::First),
        number: Some(Number::Singular),
        tense: Some(Tense::Present),
        mood: None,
        voice: Some(Voice::Active),
    };
    let v_clone = v.clone();
    assert_eq!(format!("{:?}", v), format!("{:?}", v_clone));

    // ParticipleConstituent
    let p = ParticipleConstituent {
        verb_lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        tense: Tense::Present,
        voice: Voice::Active,
        case: Case::Nominative,
        gender: Gender::Masculine,
        number: Number::Singular,
    };
    let p_clone = p.clone();
    assert_eq!(format!("{:?}", p), format!("{:?}", p_clone));

    // Constituent
    let c = Constituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        normalized: SmolStr::new("test"),
        case: Case::Nominative,
        number: Some(Number::Singular),
        gender: Some(Gender::Masculine),
        person: None,
    };
    let c_clone = c.clone();
    assert_eq!(format!("{:?}", c), format!("{:?}", c_clone));

    // Literal
    let l_str = Literal::String("s".into());
    let l_num = Literal::Number(42);
    let l_bool = Literal::Boolean(true);

    assert_eq!(format!("{:?}", l_str.clone()), format!("{:?}", l_str));
    assert_eq!(format!("{:?}", l_num.clone()), format!("{:?}", l_num));
    assert_eq!(format!("{:?}", l_bool.clone()), format!("{:?}", l_bool));

    // AssembledStatement
    let stmt = AssembledStatement {
        subject: Some(c),
        verb: Some(v),
        participles: vec![p],
        literals: vec![l_str],
        ..Default::default()
    };
    let stmt_clone = stmt.clone();
    assert_eq!(format!("{:?}", stmt), format!("{:?}", stmt_clone));
}
