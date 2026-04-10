use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn main() {
    let source = "ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.";
    let ast = parse(source).unwrap();
    println!("AST: {:#?}", ast);
    let analyzed = analyze_program(&ast);
    println!("Analyzed: {:#?}", analyzed);
}
