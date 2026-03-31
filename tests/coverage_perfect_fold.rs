#![allow(missing_docs)]
use glossa::*;

#[test]
fn test_perfect_fold_coverage() {
    // This test ensures that the "Fold" pattern logic in `src/semantic/patterns.rs`
    // is covered for Perfect tense participles.
    // We use "συλλεγμενα" (constructed perfect passive participle of συλλέγω)
    // to trigger the fold logic with Tense::Perfect.
    //
    // The stem "συλλεγ" triggers the fold detection.
    // The ending "μενα" triggers Tense::Perfect.
    //
    // We verify that it generates a .fold() call without unsafe memoization.

    let source = "[1, 2, 3] συλλεγμενα εις αθροισμα λεγε.";
    let ast = parser::parse(source).unwrap();
    let analyzed = semantic::analyze_program(&ast);

    if let Err(e) = &analyzed {
        panic!("Analysis failed: {:?}", e);
    }
    let analyzed = analyzed.unwrap();
    let code = codegen::generate_rust(&analyzed);

    println!("Generated code:\n{}", code);

    // Remove whitespace
    let code_clean = code.replace(" ", "").replace("\n", "");

    // Verify it generated a fold
    assert!(
        code_clean.contains(".fold("),
        "Should generate .fold() for συλλεγ- participle"
    );

    // Safety check: It should NOT contain the unsafe Memoization pattern
    assert!(
        !code_clean.contains("RefCell::new(None)"),
        "Should NOT use memoization for fold"
    );
}
