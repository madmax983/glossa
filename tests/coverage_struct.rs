use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_struct_fields_coverage() {
    let code = "
    α νέον Σημεῖον 10 20 ἔστω.
    ";

    let ast = parse(code).unwrap();
    let err = analyze_program(&ast).unwrap_err();
    assert_eq!(err.to_string(), "Ἄγνωστον ὄνομα: σημειον");

    let code2 = "
    εἶδος Σημεῖον ὁρίζειν { χ ἀριθμοῦ. ψ ἀριθμοῦ. }.
    α νέον Σημεῖον 10 20 ἔστω.
    ";

    let ast2 = parse(code2).unwrap();
    let analyzed = analyze_program(&ast2).unwrap();
    assert_eq!(analyzed.statements.len(), 2);
}
