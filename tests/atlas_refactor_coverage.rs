use glossa::morphology::analyze;
use glossa::semantic::{Assembler, AssembledStatement};

#[test]
fn test_assembled_statement_derive_coverage() {
    // Cover #[derive(Clone, Debug, Default)] for AssembledStatement
    let stmt = AssembledStatement::default();
    let cloned = stmt.clone();
    let debug_str = format!("{:?}", cloned);
    assert!(debug_str.contains("AssembledStatement"));

    // Cover internal fields being None/Empty by default
    assert!(stmt.subject.is_none());
    assert!(stmt.nominatives.is_empty());
}

#[test]
fn test_assembler_special_markers_coverage() {
    let mut asm = Assembler::new();

    // "μετά" (mutable marker)
    let meta = analyze("μετα"); // Preposition
    asm.feed(&meta, "μετά").unwrap();
    let stmt = asm.finalize().unwrap();
    assert!(stmt.has_mutable_marker);

    // "ἐν" (containment)
    let en = analyze("εν"); // Preposition
    asm.feed(&en, "ἐν").unwrap();
    let stmt2 = asm.finalize().unwrap();
    assert!(stmt2.has_containment_preposition);

    // "κατά" (delimiter)
    let kata = analyze("κατα"); // Preposition
    asm.feed(&kata, "κατά").unwrap();
    let stmt3 = asm.finalize().unwrap();
    assert!(stmt3.has_delimiter_preposition);
}

#[test]
fn test_assembler_method_verbs_join_coverage() {
    let mut asm = Assembler::new();

    // 1. Subject: "list"
    let list = analyze("λιστη");
    asm.feed(&list, "λίστη").unwrap();

    // 2. Delimiter Preposition: "κατά"
    let kata = analyze("κατα");
    asm.feed(&kata, "κατά").unwrap();

    // 3. Delimiter Literal: ","
    asm.feed_string(",".to_string()).unwrap();

    // 4. Join Verb: "ἑνοῦνται"
    let join = analyze("ενουνται");
    asm.feed(&join, "ἑνοῦνται").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.string_method, Some(("join".to_string(), ",".to_string())));
}

#[test]
fn test_assembler_arithmetic_operators_coverage() {
    let mut asm = Assembler::new();

    // "ἄθροισμα" (sum -> +)
    let sum = analyze("αθροισμα");
    asm.feed(&sum, "ἄθροισμα").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(!stmt.operators.is_empty());
}

#[test]
fn test_assembler_boolean_and_coverage() {
    let mut asm = Assembler::new();

    // "καί" (and)
    let and = analyze("και");
    asm.feed(&and, "καί").unwrap();

    let stmt = asm.finalize().unwrap();
    assert!(!stmt.operators.is_empty());
}

#[test]
fn test_assembler_numeral_coverage() {
    let mut asm = Assembler::new();

    // "πέντε" (5)
    let five = analyze("πεντε");
    asm.feed(&five, "πέντε").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.literals.len(), 1);
}
