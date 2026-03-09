fn main() {
    let code = "9223372036854775807 μὴ ἔστω ἄλφα.
1 ἔστω βῆτα.
ἄλφα 1 ἀφαίρεσις ἔστω γάμμα.
γάμμα μὴ λέγε.";
    let ast = glossa::parser::parse(code).unwrap();
    let program = glossa::semantic::analyze_program(&ast).unwrap();
    let mut interp = glossa::tools::interpreter::Interpreter::new();
    let res = interp.run(&program);
    println!("{:?}", res);
}
