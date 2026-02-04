use glossa::semantic::Assembler;

#[test]
fn test_assembler_string_literal_bypass() {
    let mut asm = Assembler::new();

    // The limit is 1000 tokens
    // We attempt to feed 1100 string literals
    for i in 0..1100 {
        // We attempt to feed 1100 string literals
        // We ignore the result here because we want to see how many stick
        let _ = asm.feed_string(format!("string_{}", i));
    }

    let stmt = asm.finalize().unwrap();

    // This assertion should FAIL if the vulnerability exists
    assert!(
        stmt.literals.len() <= 1000,
        "DoS vulnerability: Accepted {} tokens, limit is 1000",
        stmt.literals.len()
    );
}
