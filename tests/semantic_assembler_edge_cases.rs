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
    // REMOVED: Since "split" is now correctly identified as a verb when the pattern match fails,
    // adding another verb would cause a DoubleVerb error.
    // let verb = analyze("λεγε");
    // asm.feed(&verb, "λέγε").unwrap();

    let stmt = asm.finalize().unwrap();

    // If the literal was consumed by the failed split pattern match, it will be missing.
    // If it was preserved, it should be in stmt.literals.
    assert!(
        !stmt.literals.is_empty(),
        "Literal should not be consumed if split pattern fails to match due to missing subject"
    );

    // Also verify that "split" was captured as the verb
    assert!(stmt.verb.is_some(), "Split should be captured as the verb");
    assert_eq!(stmt.verb.unwrap().lemma, "σχιζω");
}

#[test]
fn test_split_verb_not_ignored_without_delimiter() {
    let mut asm = Assembler::new();

    // Feed subject "word"
    let subj = analyze("λογος"); // "word"
    asm.feed(&subj, "λόγος").unwrap();

    // Feed "splits" (σχίζει) without "by" (κατά) and delimiter string
    // normalized: σχιζει
    // This should now be treated as a normal verb because the split pattern didn't match!
    let split_verb = analyze("σχιζει");
    asm.feed(&split_verb, "σχίζει").unwrap();

    let stmt = asm.finalize();

    match stmt {
        Ok(s) => {
            // Now we expect the verb to be present!
            assert!(s.verb.is_some(), "Verb should be present (treated as normal verb) when split pattern fails");
            let verb = s.verb.unwrap();
            assert_eq!(verb.original, "σχίζει");
            assert!(s.string_method.is_none(), "String method should be None");
        },
        Err(e) => {
             panic!("Should not error: {:?}", e);
        }
    }
}

#[test]
fn test_ordinal_not_ignored_without_subject() {
    let mut asm = Assembler::new();

    // Feed "first" (πρῶτον) - Ordinal
    // normalized: πρωτον
    // Since there is no subject yet, it should fall through and be treated as an Adjective
    let first = analyze("πρωτον");
    asm.feed(&first, "πρῶτον").unwrap();

    // Feed "man" (ἄνθρωπος) - Subject
    let man = analyze("ανθρωπος");
    asm.feed(&man, "ἄνθρωπος").unwrap();

    // Feed "is" (ἐστί) - Verb
    let is_verb = analyze("εστι");
    asm.feed(&is_verb, "ἐστί").unwrap();

    let stmt = asm.finalize().unwrap();

    assert!(stmt.subject.is_some(), "Subject should be present");
    assert_eq!(stmt.subject.unwrap().original, "ἄνθρωπος");

    // "first" should be in adjectives now!
    assert!(!stmt.adjectives.is_empty(), "Adjectives should NOT be empty; 'first' should be captured");
    assert_eq!(stmt.adjectives[0].original, "πρῶτον");

    assert!(stmt.index_accesses.is_empty(), "Index accesses should be empty");
}

#[test]
fn test_length_property_not_ignored_without_subject() {
    let mut asm = Assembler::new();

    // Feed "length" (μῆκος) - Noun
    // normalized: μηκος
    // Since there is no subject, it should fall through and be treated as a Noun (Subject/Object)
    let len = analyze("μηκος");
    asm.feed(&len, "μῆκος").unwrap();

    // Feed "is" (ἐστί)
    let is_verb = analyze("εστι");
    asm.feed(&is_verb, "ἐστί").unwrap();

    // Feed "5"
    asm.feed_number(5);

    let stmt = asm.finalize().unwrap();

    // "length" should be the subject now!
    assert!(stmt.subject.is_some(), "Subject should be present (length)");
    assert_eq!(stmt.subject.unwrap().lemma, "μηκος");

    assert!(stmt.property_accesses.is_empty(), "Property accesses should be empty");
}
