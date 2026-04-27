use glossa::parser::parse;
use glossa::semantic::assembly::Assembler;
use glossa::morphology::analyze_all;

#[test]
fn test_print_ast() {
    let mut asm = Assembler::new();
    let words = vec!["ωου", "alpha"];
    for w in words {
        let analyses = analyze_all(w);
        asm.feed(&analyses[0], w).unwrap();
    }
    let stmt = asm.finalize().unwrap();
    println!("{:#?}", stmt);
    panic!("look at me");
}
