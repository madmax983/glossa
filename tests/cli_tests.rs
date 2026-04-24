use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn main() {
    let source = "ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.";
    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);
    println!("{:?}", result);
}
