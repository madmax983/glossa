use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn debug_test() {
    let code = "ξ ἀληθές ἔστω. ψ πέντε ἔστω. ψ ξ! ἔστω.";
    let ast = parse(code).unwrap();
    let program = analyze_program(&ast).unwrap();
    println!("{:#?}", program);
}
