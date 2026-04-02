use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn debug_test() {
    let code = "ξ 1 ἔστω. π λέγε · ξ δός.";
    let ast = parse(code).unwrap();
    let program = analyze_program(&ast).unwrap();
    println!("{:#?}", program);
}
