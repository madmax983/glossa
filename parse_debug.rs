use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn main() {
    let code = "ξ πέντε ἔστω. ξ;";
    let ast = parse(code).unwrap();
    let program = analyze_program(&ast).unwrap();
    println!("{:#?}", program);
}
