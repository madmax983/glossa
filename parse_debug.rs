use glossa::semantic::assemble_statement;
use glossa::parser::parse;

fn main() {
    let src = "μηδὲν ᾖ";
    let stmt = &parse(src).unwrap().statements[0];
    let asm = assemble_statement(stmt).unwrap();
    println!("{:?}", asm);
}
