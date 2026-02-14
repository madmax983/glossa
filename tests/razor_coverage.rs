//! Razor Coverage Tests
//!
//! These tests explicitly exercise the public API of the refactored AST and Error modules
//! to ensure high code coverage (satisfying the "Razor" persona's strict requirements).
//!
//! We use the "Nuclear Option" of constructing all enum variants and calling all methods
//! to ensure no code path is left behind.

use glossa::ast::{
    Clause, Expr, Statement, TestDecl, TraitDef,
    TraitImplDef, TypeDef, Word,
};
use glossa::errors::{
    case_mismatch, gender_mismatch, immutable_assignment, number_mismatch, type_mismatch,
    undefined_variable, AssemblyError, GlossaError,
};
use glossa::morphology::{Case, Gender, Number, Person};
use miette::SourceSpan;

#[test]
fn test_statement_expressions_iterator() {
    // 1. Regular Statement
    let regular = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::NumberLiteral(1)],
        }],
        is_query: false,
        is_propagate: false,
    };
    assert_eq!(regular.expressions().count(), 1);

    // 2. Type Definition (should be empty)
    let type_def = Statement::TypeDefinition(TypeDef {
        name: Word::new("Test"),
        fields: vec![],
    });
    assert_eq!(type_def.expressions().count(), 0);

    // 3. Trait Definition (should be empty)
    let trait_def = Statement::TraitDefinition(TraitDef {
        name: Word::new("Trait"),
        methods: vec![],
    });
    assert_eq!(trait_def.expressions().count(), 0);

    // 4. Trait Impl (should be empty)
    let trait_impl = Statement::TraitImpl(TraitImplDef {
        type_name: Word::new("Test"),
        trait_name: Word::new("Trait"),
        methods: vec![],
    });
    assert_eq!(trait_impl.expressions().count(), 0);

    // 5. Test Declaration (should be empty)
    let test_decl = Statement::TestDeclaration(TestDecl {
        name: "test".into(),
        body: vec![],
    });
    assert_eq!(test_decl.expressions().count(), 0);
}

#[test]
fn test_statement_is_query() {
    let regular = Statement::Regular {
        clauses: vec![],
        is_query: true,
        is_propagate: false,
    };
    assert!(regular.is_query());

    let regular_false = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: false,
    };
    assert!(!regular_false.is_query());

    // Other types always false
    let type_def = Statement::TypeDefinition(TypeDef {
        name: Word::new("T"),
        fields: vec![],
    });
    assert!(!type_def.is_query());
}

#[test]
fn test_statement_is_propagate() {
    let regular = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: true,
    };
    assert!(regular.is_propagate());

    let regular_false = Statement::Regular {
        clauses: vec![],
        is_query: false,
        is_propagate: false,
    };
    assert!(!regular_false.is_propagate());

    // Other types always false
    let type_def = Statement::TypeDefinition(TypeDef {
        name: Word::new("T"),
        fields: vec![],
    });
    assert!(!type_def.is_propagate());
}

#[test]
fn test_statement_clauses() {
    let clauses = vec![Clause {
        expressions: vec![Expr::NumberLiteral(1)],
    }];
    let regular = Statement::Regular {
        clauses: clauses.clone(),
        is_query: false,
        is_propagate: false,
    };
    assert_eq!(regular.clauses(), &clauses);

    // Other types return empty slice
    let type_def = Statement::TypeDefinition(TypeDef {
        name: Word::new("T"),
        fields: vec![],
    });
    assert!(type_def.clauses().is_empty());
}

#[test]
fn test_error_category_greek() {
    // 1. Parse Error
    let parse = GlossaError::parse("msg");
    assert_eq!(parse.category_greek(), "Σύνταξις");

    // 2. Semantic Error
    let semantic = GlossaError::semantic("msg");
    assert_eq!(semantic.category_greek(), "Σημασία");

    // 3. Type Error
    let type_err = GlossaError::type_error("msg");
    assert_eq!(type_err.category_greek(), "Τύπος");

    // 4. Undefined Name
    let undefined = GlossaError::undefined("name");
    assert_eq!(undefined.category_greek(), "Ὄνομα");

    // 5. Agreement Error
    let agreement = GlossaError::agreement("msg");
    assert_eq!(agreement.category_greek(), "Συμφωνία");

    // 6. Codegen Error
    let codegen = GlossaError::codegen("msg");
    assert_eq!(codegen.category_greek(), "Κῶδιξ");

    // 7. IO Error
    let io = GlossaError::io("msg");
    assert_eq!(io.category_greek(), "Ἀρχεῖον");

    // 8. Limit Exceeded
    let limit = GlossaError::LimitExceeded {
        resource: "depth".into(),
        max: 100,
    };
    assert_eq!(limit.category_greek(), "Όριον");

    // 9. Assembly Error
    let assembly = GlossaError::AssemblyError(AssemblyError::MissingVerb);
    assert_eq!(assembly.category_greek(), "Συναρμογή");
}

#[test]
fn test_error_constructors() {
    // parse_with_source
    let _ = GlossaError::parse_with_source("msg", "src", SourceSpan::new(0usize.into(), 0usize.into()));

    // Verify debug formatting
    let err = GlossaError::parse("test");
    let debug = format!("{:?}", err);
    assert!(debug.contains("ParseError"));
}

#[test]
fn test_assembly_error_variants_display() {
    // Construct all variants and check Display impl (for coverage)
    let errors = vec![
        AssemblyError::DoubleSubject,
        AssemblyError::DoubleObject,
        AssemblyError::DoubleIndirect,
        AssemblyError::DoubleVerb,
        AssemblyError::MissingVerb,
        AssemblyError::SubjectVerbDisagreement {
            subject: (Some(Person::Third), Some(Number::Singular)),
            verb: (Some(Person::Third), Some(Number::Plural)),
        },
        AssemblyError::GenderMismatch {
            word1: "w1".into(),
            gender1: Gender::Masculine,
            word2: "w2".into(),
            gender2: Gender::Feminine,
        },
        AssemblyError::LimitExceeded {
            resource: "adjectives".into(),
            max: 5,
        },
    ];

    for err in errors {
        let _ = format!("{}", err); // Trigger Display
        let _ = format!("{:?}", err); // Trigger Debug
    }
}

#[test]
fn test_error_helper_functions() {
    // 1. type_mismatch
    let msg = type_mismatch("A", "B");
    assert!(msg.contains("Ἐδόκει A εὑρεῖν, ἀλλ' εὗρον B"));

    // 2. undefined_variable
    let msg = undefined_variable("x");
    assert!(msg.contains("Οὐκ οἶδα τὸ ὄνομα «x»"));

    // 3. immutable_assignment
    let msg = immutable_assignment("x");
    assert!(msg.contains("Τὸ «x» ἀμετάβλητόν ἐστιν"));

    // 4. gender_mismatch
    let msg = gender_mismatch("w1", Gender::Masculine, "w2", Gender::Feminine);
    assert!(msg.contains("οὐ συμφωνεῖ"));

    // 5. number_mismatch
    let msg = number_mismatch("w1", Number::Singular, "w2", Number::Plural);
    assert!(msg.contains("οὐ συμφωνεῖ"));

    // 6. case_mismatch
    let msg = case_mismatch("w1", Case::Nominative, "w2", Case::Accusative);
    assert!(msg.contains("οὐ συμφωνεῖ"));
}
