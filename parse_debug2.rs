use glossa::semantic::assemble_statement;
use glossa::parser::parse;
use glossa::semantic::conversion::classify_assembled_statement;

fn main() {
    let src = "μηδὲν ᾖ";
    let stmt = &parse(src).unwrap().statements[0];
    let asm = assemble_statement(stmt).unwrap();
    let mut scope = glossa::semantic::Scope::new();
    let analyzed = classify_assembled_statement(&asm, &mut scope);
    println!("{:?}", analyzed);
}
