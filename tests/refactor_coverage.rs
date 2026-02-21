use glossa::ast::{Clause, Expr, Statement};
use glossa::codegen::{generate_rust_file, generate_statement_code};
use glossa::errors::{AssemblyError, GlossaError, help};
use glossa::morphology::{Gender, Number, Person};
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
};

// --- Errors ---

#[test]
fn test_assembly_error_formatting() {
    let err = AssemblyError::DoubleSubject;
    assert!(err.to_string().contains("Διπλοῦν ὑποκείμενον"));

    let err = AssemblyError::DoubleObject;
    assert!(err.to_string().contains("Διπλοῦν ἀντικείμενον"));

    let err = AssemblyError::DoubleIndirect;
    assert!(err.to_string().contains("Διπλοῦν ἔμμεσον αντικείμενον"));

    let err = AssemblyError::DoubleVerb;
    assert!(err.to_string().contains("Διπλοῦν ῥῆμα"));

    let err = AssemblyError::MissingVerb;
    assert!(err.to_string().contains("Ῥῆμα οὐχ εὑρέθη"));

    let err = AssemblyError::SubjectVerbDisagreement {
        subject: (Some(Person::Third), Some(Number::Singular)),
        verb: (Some(Person::Third), Some(Number::Plural)),
    };
    assert!(err.to_string().contains("Ἀσυμφωνία"));

    let err = AssemblyError::GenderMismatch {
        word1: "word1".to_string(),
        gender1: Gender::Masculine,
        word2: "word2".to_string(),
        gender2: Gender::Feminine,
    };
    assert!(err.to_string().contains("Ἀσυμφωνία γένους"));

    let err = AssemblyError::LimitExceeded {
        resource: "Test".to_string(),
        max: 10,
    };
    assert!(err.to_string().contains("Ὑπέρβασις ὁρίου"));
}

#[test]
fn test_glossa_error_formatting() {
    let err = GlossaError::parse("parse");
    assert_eq!(err.category_greek(), "Σύνταξις");

    let err = GlossaError::semantic("semantic");
    assert_eq!(err.category_greek(), "Σημασία");

    let err = GlossaError::undefined("name");
    assert_eq!(err.category_greek(), "Ὄνομα");

    let err = GlossaError::agreement("agreement");
    assert_eq!(err.category_greek(), "Συμφωνία");

    let err = GlossaError::codegen("codegen");
    assert_eq!(err.category_greek(), "Κῶδιξ");

    let err = GlossaError::LimitExceeded {
        resource: "res".into(),
        max: 10,
    };
    assert_eq!(err.category_greek(), "Όριον");

    let err = GlossaError::AssemblyError(AssemblyError::MissingVerb);
    assert_eq!(err.category_greek(), "Συναρμογή");

    // Check with source constructor
    let _ = GlossaError::parse_with_source("parse", "source", (0, 5).into());
}

#[test]
fn test_help_messages() {
    assert!(help::BINDING.contains("Χρῆσις"));
    assert!(help::PRINT.contains("Χρῆσις"));
    assert!(help::CASES.contains("Πτώσεις"));
}

// --- AST ---

#[test]
fn test_ast_statement_methods() {
    let stmt = Statement::Regular {
        clauses: vec![Clause {
            expressions: vec![Expr::NumberLiteral(1)],
        }],
        is_query: true,
        is_propagate: true,
    };

    assert!(stmt.is_query());
    assert!(stmt.is_propagate());
    assert_eq!(stmt.clauses().len(), 1);

    // Test expressions()
    let exprs: Vec<&Expr> = stmt.expressions().collect();
    assert_eq!(exprs.len(), 1);
    assert!(matches!(exprs[0], Expr::NumberLiteral(1)));

    // Test dummy statements
    let type_def = Statement::TypeDefinition(glossa::ast::TypeDef {
        name: glossa::ast::Word::new("Test"),
        fields: vec![],
    });
    assert!(!type_def.is_query());
    assert!(!type_def.is_propagate());
    assert!(type_def.clauses().is_empty());
    assert!(type_def.expressions().next().is_none());
}

// --- Codegen ---

#[test]
fn test_codegen_program() {
    let program = AnalyzedProgram {
        statements: vec![AnalyzedStatement::Print(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral("Hello".to_string()),
            glossa_type: GlossaType::String,
        }])],
        scope: Scope::new(),
    };

    let code = generate_rust_file(&program);
    assert!(code.contains("#![allow(non_snake_case, unused_imports)]"));
    assert!(code.contains("println"));
}

#[test]
fn test_codegen_uses_collections() {
    // A program that uses a map
    let program = AnalyzedProgram {
        statements: vec![AnalyzedStatement::Binding {
            name: "m".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::CollectionNew {
                    collection_type: "HashMap".to_string(),
                },
                glossa_type: GlossaType::Map(
                    Box::new(GlossaType::Unknown),
                    Box::new(GlossaType::Unknown),
                ),
            },
            mutable: true,
        }],
        scope: Scope::new(),
    };

    let code = generate_rust_file(&program);
    // quote! might affect formatting (e.g. "std :: collections"), so check for parts
    assert!(code.contains("std :: collections"));
    assert!(code.contains("HashMap"));
    assert!(code.contains("HashSet"));
}

#[test]
fn test_codegen_statement() {
    let stmt = AnalyzedStatement::Print(vec![AnalyzedExpr {
        expr: AnalyzedExprKind::NumberLiteral(42),
        glossa_type: GlossaType::Number,
    }]);

    let code = generate_statement_code(&stmt);
    assert!(code.contains("println"));
    assert!(code.contains("42"));
}

// --- Semantic Model ---

#[test]
fn test_glossatype_methods() {
    assert_eq!(GlossaType::Number.to_greek(), "ἀριθμός");
    assert_eq!(GlossaType::String.to_greek(), "ὄνομα");
    assert_eq!(GlossaType::Boolean.to_greek(), "ἀληθές");
    assert_eq!(GlossaType::Unit.to_greek(), "οὐδέν");

    let list_ty = GlossaType::List(Box::new(GlossaType::Number));
    assert_eq!(list_ty.to_greek(), "λίστη");
    assert!(list_ty.is_compatible(&list_ty));

    let map_ty = GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number));
    assert_eq!(map_ty.to_greek(), "χάρτης");

    let opt_ty = GlossaType::Option(Box::new(GlossaType::Number));
    assert_eq!(opt_ty.to_greek(), "εὑρεθείη");

    let res_ty = GlossaType::Result(Box::new(GlossaType::Unit), Box::new(GlossaType::String));
    assert_eq!(res_ty.to_greek(), "ἀποτέλεσμα");
}
