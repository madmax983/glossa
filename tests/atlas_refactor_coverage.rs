use glossa::morphology::analyze;
use glossa::semantic::{AssembledStatement, Assembler, AssemblyError, Constituent};

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
    asm.feed_string(",").unwrap();

    // 4. Join Verb: "ἑνοῦνται"
    let join = analyze("ενουνται");
    asm.feed(&join, "ἑνοῦνται").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(
        stmt.string_method,
        Some(("join".to_string(), ",".to_string()))
    );
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

#[test]
fn test_assembler_set_flags_coverage() {
    let mut asm = Assembler::new();
    asm.set_query(true);
    asm.set_propagate(true);

    let stmt = asm.finalize().unwrap();
    assert!(stmt.is_query);
    assert!(stmt.is_propagate);
}

#[test]
fn test_assembler_has_content_coverage() {
    let mut asm = Assembler::new();
    assert!(!asm.has_content());

    asm.feed_string("test").unwrap();
    assert!(asm.has_content());

    let _ = asm.finalize().unwrap();
    assert!(!asm.has_content());
}

#[test]
fn test_assembler_method_verbs_split_coverage() {
    let mut asm = Assembler::new();

    // 1. Subject: "string"
    let string_noun = analyze("λογος");
    asm.feed(&string_noun, "λόγος").unwrap();

    // 2. Delimiter Preposition: "κατά"
    let kata = analyze("κατα");
    asm.feed(&kata, "κατά").unwrap();

    // 3. Delimiter Literal: "."
    asm.feed_string(".").unwrap();

    // 4. Split Verb: "σχίζεται"
    let split = analyze("σχιζεται");
    asm.feed(&split, "σχίζεται").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(
        stmt.string_method,
        Some(("split".to_string(), ".".to_string()))
    );
}

#[test]
fn test_assembler_ordinal_index_coverage() {
    let mut asm = Assembler::new();

    // 1. Subject: "array" (use known noun "λιστη")
    let array = analyze("λιστη");
    asm.feed(&array, "λίστη").unwrap();

    // 2. Ordinal: "first" (should map to index 0)
    let first = analyze("πρωτον");
    asm.feed(&first, "πρῶτον").unwrap();

    let stmt = asm.finalize().unwrap();
    assert_eq!(stmt.index_accesses.len(), 1);
    // Subject should be consumed
    assert!(stmt.subject.is_none());
}

#[test]
fn test_assembler_error_cases_coverage() {
    let mut asm = Assembler::new();

    // Double Verb
    let verb1 = analyze("λεγει");
    asm.feed(&verb1, "λέγει").unwrap();
    let result = asm.feed(&verb1, "λέγει");
    assert!(matches!(result, Err(AssemblyError::DoubleVerb)));
    let _ = asm.finalize();

    // Double Object
    let obj = analyze("λογον");
    asm.feed(&obj, "λόγον").unwrap();
    let result = asm.feed(&obj, "λόγον");
    assert!(matches!(result, Err(AssemblyError::DoubleObject)));
    let _ = asm.finalize();

    // Double Indirect
    let ind = analyze("ανθρωπω");
    asm.feed(&ind, "ἀνθρώπῳ").unwrap();
    let result = asm.feed(&ind, "ἀνθρώπῳ");
    assert!(matches!(result, Err(AssemblyError::DoubleIndirect)));
}

#[test]
fn test_constituent_derive_coverage() {
    use glossa::morphology::Case;
    use smol_str::SmolStr;

    let c = Constituent {
        lemma: SmolStr::new("test"),
        original: SmolStr::new("test"),
        case: Case::Nominative,
        number: None,
        gender: None,
        person: None,
    };

    let cloned = c.clone();
    let debug = format!("{:?}", cloned);
    assert!(debug.contains("test"));
}
