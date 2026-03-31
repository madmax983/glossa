#![allow(missing_docs)]
use glossa::*;

#[test]
fn test_perfect_passive_map_coverage() {
    // This test ensures that Perfect Passive participles (e.g. "γεγραμμενα")
    // trigger the iterator map logic in `src/semantic/patterns.rs`.
    // Previously, this logic was unreachable because it only checked for Voice::Middle.
    //
    // By enabling this, we ensure that our security fix (downgrading Perfect tense
    // from Memoize to Borrow for iterators) is actually exercised and covered.

    // [1, 2, 3] γεγραμμενα λέγε.
    // "γεγραμμενα" is Perfect Passive Participle of "γραφω".
    // Should be interpreted as an identity map (default behavior) or similar.
    let source = "[1, 2, 3] γεγραμμενα λέγε.";
    let ast = parser::parse(source).unwrap();
    let analyzed = semantic::analyze_program(&ast);

    if let Err(e) = &analyzed {
        panic!("Analysis failed: {:?}", e);
    }
    let analyzed = analyzed.unwrap();
    let code = codegen::generate_rust(&analyzed);

    println!("Generated code:\n{}", code);

    // Remove whitespace to make matching robust against formatting
    let code_clean = code.replace(" ", "").replace("\n", "");

    // We expect a .map() call.
    assert!(
        code_clean.contains(".map("),
        "Should generate .map() for perfect passive participle"
    );

    // Safety check: It should NOT contain the unsafe Memoization pattern (RefCell/OnceCell)
    // because we downgraded it to Borrow in patterns.rs.
    assert!(
        !code_clean.contains("RefCell::new(None)"),
        "Should NOT use memoization for iterator map"
    );
}
