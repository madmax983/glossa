use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn debug_test() {
    let code = "ξ πέντε ἔστω. ψ ξ 2 ἄθροισμα ἔστω.";
    let ast = parse(code).unwrap();
    let program = analyze_program(&ast).unwrap();
    println!("{:#?}", program);
}
