use glossa::morphology::analyze;
use glossa::semantic::assembler::Assembler;

#[test]
fn test_split_verb_consumes_literal_without_subject() {
    let mut asm = Assembler::new();

    // 1. Feed "κατά" (delimiter preposition)
    let kata = analyze("κατα"); // Should be preposition
    asm.feed(&kata, "κατά").unwrap();

    // 2. Feed string literal " "
    asm.feed_string(" ".to_string());

    // 3. Feed "σχίζεται" (split verb)
    // This triggers check_method_verbs
    let split = analyze("σχιζεται");
    asm.feed(&split, "σχίζεται").unwrap();

    // At this point, if the bug exists:
    // - check_method_verbs returned true (so it was "handled")
    // - pending_literals.pop() was called and consumed " "
    // - pending_subject was None, so no property access was created.

    // 4. Feed "λόγος" (subject)
    let subject = analyze("λογος");
    asm.feed(&subject, "λόγος").unwrap();

    // 5. Feed "λέγε" (print verb)
    let verb = analyze("λεγε");
    asm.feed(&verb, "λέγε").unwrap();

    let stmt = asm.finalize().unwrap();

    // If the literal was consumed by the failed split pattern match, it will be missing.
    // If it was preserved, it should be in stmt.literals.
    assert!(
        !stmt.literals.is_empty(),
        "Literal should not be consumed if split pattern fails to match due to missing subject"
    );
}
