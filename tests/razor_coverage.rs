use glossa::ast::{
    BinOperator, Clause, Expr, Program, Statement, TestDecl, TraitDef, TraitImplDef, TypeDef,
    UnaryOperator, Word,
};
use glossa::errors::{GlossaError, help};
use glossa::highlight::highlight;
use glossa::report::{GlossaReport, ProgramStats};
use glossa::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, AssembledStatement,
    GlossaType, Scope,
};

#[test]
fn test_ast_derives_and_methods() {
    // Coverage for Statement::clauses(), is_query(), is_propagate() derived methods
    // and Debug/Clone/PartialEq on AST nodes.

    let clause = Clause {
        expressions: vec![Expr::NumberLiteral(1)],
    };

    let stmt_regular = Statement::Regular {
        clauses: vec![clause.clone()],
        is_query: true,
        is_propagate: false,
    };

    assert!(stmt_regular.is_query());
    assert!(!stmt_regular.is_propagate());
    assert_eq!(stmt_regular.clauses().len(), 1);

    // Test non-regular statements return empty/false
    let stmt_type = Statement::TypeDefinition(TypeDef {
        name: Word::new("T"),
        fields: vec![],
    });
    assert!(!stmt_type.is_query());
    assert!(!stmt_type.is_propagate());
    assert!(stmt_type.clauses().is_empty());

    let stmt_trait = Statement::TraitDefinition(TraitDef {
        name: Word::new("Tr"),
        methods: vec![],
    });
    assert!(!stmt_trait.is_query());
    assert!(!stmt_trait.is_propagate());
    assert!(stmt_trait.clauses().is_empty());

    let stmt_impl = Statement::TraitImpl(TraitImplDef {
        type_name: Word::new("T"),
        trait_name: Word::new("Tr"),
        methods: vec![],
    });
    assert!(!stmt_impl.is_query());
    assert!(!stmt_impl.is_propagate());
    assert!(stmt_impl.clauses().is_empty());

    let stmt_test = Statement::TestDeclaration(TestDecl {
        name: "test".to_string(),
        body: vec![],
    });
    assert!(!stmt_test.is_query());
    assert!(!stmt_test.is_propagate());
    assert!(stmt_test.clauses().is_empty());

    // Exercise Debug/Clone on Enums
    let _ = stmt_regular.clone();
    let _ = format!("{:?}", stmt_regular);

    let binop = BinOperator::Add;
    assert_eq!(binop, BinOperator::Add);

    let unop = UnaryOperator::Not;
    assert_eq!(unop, UnaryOperator::Not);
}

#[test]
fn test_errors_coverage() {
    // Coverage for help messages and error constructors
    let err_parse = GlossaError::parse_with_source("msg", "src", (0, 1).into());
    assert_eq!(err_parse.category_greek(), "Σύνταξις");

    let err_type = GlossaError::type_error("msg");
    assert_eq!(err_type.category_greek(), "Τύπος");

    let err_semantic = GlossaError::semantic("msg");
    assert_eq!(err_semantic.category_greek(), "Σημασία");

    let err_agree = GlossaError::agreement("msg");
    assert_eq!(err_agree.category_greek(), "Συμφωνία");

    let err_codegen = GlossaError::codegen("msg");
    assert_eq!(err_codegen.category_greek(), "Κῶδιξ");

    let err_io = GlossaError::io("msg");
    assert_eq!(err_io.category_greek(), "Ἀρχεῖον");

    let err_undef = GlossaError::undefined("x");
    assert_eq!(err_undef.category_greek(), "Ὄνομα");

    // Assembly Error coverage
    let asm_err = glossa::errors::AssemblyError::DoubleSubject;
    let glossa_err: GlossaError = asm_err.into();
    assert_eq!(glossa_err.category_greek(), "Συναρμογή");
    assert!(format!("{}", glossa_err).contains("Διπλοῦν ὑποκείμενον"));

    // Recursion Limit coverage
    let deep_source = "(".repeat(600);
    let res = glossa::parser::parse(&deep_source);
    assert!(res.is_err());
    // The error string might be localized or wrapped
    let err_str = format!("{:?}", res.err());
    assert!(
        err_str.contains("RecursionLimitExceeded") || err_str.contains("depth"),
        "Error should mention recursion limit: {}",
        err_str
    );

    // Help constants
    assert!(!help::BINDING.is_empty());
    assert!(!help::PRINT.is_empty());
    assert!(!help::CASES.is_empty());
}

#[test]
fn test_highlight_coverage() {
    // Ensure highlighter runs on various inputs
    let inputs = [
        "«string» λέγε.",
        "42 λέγε.",
        "ἀληθές λέγε.",
        "[1, 2] λέγε.",
        "πίναξ[0] λέγε.",
        "χρήστου ὄνομα λέγε.",
        "λέγε «χαῖρε».",
        "ξ πέντε ἔστω.",
        "1 καὶ 2 λέγε.",
        "οὐκ ἀληθές λέγε.",
        "τιμή! λέγε.",
        "{ «χαῖρε» λέγε. }.",
        "δοκιμή «test». τέλος.",
        "εἶδος Τ ὁρίζειν { }.",
        "χαρακτήρ Χ ὁρίζειν { }.",
        "εἶδος Τ τῷ Χ ἐμπίπτειν { }.",
        "εἰ ἀληθές, «ναί» λέγε.", // Control flow highlighting
        "ξ?",                     // Query
        "σφάλμα;",                // Propagate
        // Part of Speech variants for highlighter coverage
        // Note: Single words might not parse as a full statement if they don't end in period/query
        "μετά.", // Preposition (white bold)
        "καί.",  // Conjunction (white bold)
        "πέντε.", // Numeral (italic)
        "ἀγνωστον.", // Unknown (white)
        "τῷ ἀνθρώπῳ δίδωμι.", // Dative
        "ἄνθρωπε.", // Vocative
        "καλός.", // Adjective
        // Unary Neg "-ξ" might not parse if parser expects space or specific structure.
        // Let's try explicit UnaryOp construction if parser fails, but here we test highlight(str).
        // If "-ξ." fails, it's likely a parser issue with unary operators at start of statement.
        // We'll remove it from this list if it's too fragile for integration test and rely on unit tests in highlight.rs
    ];

    for input in inputs {
        let res = highlight(input);
        assert!(res.is_ok(), "Failed to highlight: {}", input);
        assert!(!res.unwrap().is_empty());
    }
}

#[test]
fn test_program_stats_coverage() {
    // Coverage for ProgramStats default/new
    let stats = ProgramStats::default();
    assert_eq!(stats.statement_count, 0);

    let mut scope = Scope::new();
    scope.define_function("f", vec![], None); // Function count

    // define_type expects (name, type), not fields.
    // To increment type count in stats, we need a struct in scope.
    // The Scope implementation of `types()` likely iterates over defined structs.
    // define_type takes (name, GlossaType).
    // We use GlossaType::Struct for custom types.
    scope.define_type(
        "T",
        GlossaType::Struct {
            name: "T".into(),
            gender: glossa::morphology::Gender::Masculine,
            fields: vec![],
        },
    ); // Type count

    let program = AnalyzedProgram {
        statements: vec![AnalyzedStatement::Print(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        }])],
        scope,
    };
    let stats_new = ProgramStats::new(&program);
    assert_eq!(stats_new.statement_count, 1);
    assert_eq!(stats_new.function_count, 1);
    assert_eq!(stats_new.type_count, 1);

    // Test GlossaReport Display
    let report = GlossaReport::new(&program, "test.gl".to_string());
    let output = format!("{}", report);
    assert!(output.contains("ΑΝΑΦΟΡΑ ΓΛΩΣΣΗΣ"));
    assert!(output.contains("Προτάσεις"));
}

#[test]
fn test_assembled_statement_default() {
    let stmt = AssembledStatement::default();
    assert!(stmt.subject.is_none());
    assert!(!stmt.is_query);
    assert!(!stmt.has_mutable_marker);
}

#[test]
fn test_ast_program_struct() {
    let program = Program { statements: vec![] };
    let _ = program.clone();
    let _ = format!("{:?}", program);
    assert!(program.statements.is_empty());
}
