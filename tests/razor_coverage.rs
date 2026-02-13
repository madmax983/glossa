use glossa::ast::{
    BinOperator, Clause, Expr, Program, Statement, TestDecl, TraitDef, TraitImplDef, TypeDef,
    UnaryOperator, Word,
};
use glossa::errors::{GlossaError, help};
use glossa::highlight::highlight;
use glossa::report::ProgramStats;
use glossa::semantic::{AnalyzedProgram, AssembledStatement, Scope};

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
    let _ = binop.clone();
    assert_eq!(binop, BinOperator::Add);

    let unop = UnaryOperator::Not;
    let _ = unop.clone();
    assert_eq!(unop, UnaryOperator::Not);
}

#[test]
fn test_errors_coverage() {
    // Coverage for help messages and error constructors
    let _ = GlossaError::parse_with_source("msg", "src", (0, 1).into());
    let _ = GlossaError::type_error("msg");
    let _ = GlossaError::agreement("msg");
    let _ = GlossaError::codegen("msg");
    let _ = GlossaError::io("msg");

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

    let scope = Scope::new();
    let program = AnalyzedProgram {
        statements: vec![],
        scope,
    };
    let stats_new = ProgramStats::new(&program);
    assert_eq!(stats_new.statement_count, 0);
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
