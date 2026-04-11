fn main() {
    let result = glossa::parser::parse("ἄγνωστος λέγε.");
    println!("{:?}", result);
}
