fn main() {
    let source = "δοκιμή «test» .\n    1 1 ἰσοῦται.\nτέλος.";
    let program = glossa::parser::parse(source).unwrap();
    println!("{:?}", program.statements);
}
