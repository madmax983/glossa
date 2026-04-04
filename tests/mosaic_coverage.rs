#![allow(missing_docs)]
#![cfg(feature = "nova")]

use glossa::tools::mosaic::run_mosaic_inner;

#[test]
fn test_mosaic_comprehensive_coverage() {
    // Corrected test string with valid syntax
    // - Arrays: `[1, 2]`
    // - Index Access: `arr[0]`
    // - Properties: `α μῆκος`
    // - Nested Phrases: `λέγε (ὁ ἄνθρωπος).` (Phrase inside phrase)
    // - Blocks: `{ 1. }`
    // - Unwraps: `α!`
    // - String Methods: `α κατά «,» σχίζεται.`
    // - Flags: `?`, `;`, `μετά`, `ἐν`, `κατά`.

    let source = "
        [1, 2] λίστα ἔστω.
        λίστα[0] λέγε.
        α μῆκος λέγε.
        α κατά «,» σχίζεται.
        x ἐν y?
        x μετά y;
        α! λέγε.
        { 1. } λέγε.
        λέγε (ὁ ἄνθρωπος).
    ";

    let mut buffer = Vec::new();
    match run_mosaic_inner(source, &mut buffer) {
        Ok(_) => {}
        Err(e) => panic!("Mosaic failed: {:?}", e),
    }
    let output = String::from_utf8(buffer).unwrap();

    println!("Output:\n{}", output);

    // Verify Arrays
    assert!(output.contains("Arrays:"), "Missing Arrays output");

    // Verify Index Accesses
    assert!(
        output.contains("Index Accesses:"),
        "Missing Index Accesses output"
    );

    // Verify Blocks (from `{ 1. }`)
    assert!(output.contains("Blocks:"), "Missing Blocks output");

    // Verify Nested Phrases (from `(ὁ ἄνθρωπος)`)
    // If (ὁ ἄνθρωπος) is parsed as a phrase, it should appear.
    // However, if the parser treats `ὁ` and `ἄνθρωπος` as separate terms in the outer phrase, it might not nest.
    // Parentheses enforce nesting in `build_expression`.
    // So `(ὁ ἄνθρωπος)` -> `Expr::Phrase`.
    // `λέγε (ὁ ἄνθρωπος)` -> `Expr::Phrase(vec![Word, Expr::Phrase])`.
    // This should trigger `asm.feed_nested_phrase`.
    assert!(output.contains("Phrases:"), "Missing Nested Phrases output");

    // Verify Unwraps (from `α!`)
    assert!(output.contains("Unwraps:"), "Missing Unwraps output");

    // Verify Properties (μῆκος -> len)
    assert!(output.contains("Properties:"), "Missing Properties output");

    // Verify String Method (split)
    assert!(
        output.contains("Method: split(,)"),
        "Missing String Method output"
    );

    // Verify Flags
    assert!(output.contains("Query (?)"), "Missing Query flag");
    assert!(output.contains("Propagate (;)"), "Missing Propagate flag");
    assert!(output.contains("Mut (μετά)"), "Missing Mutable flag");
    assert!(output.contains("In (ἐν)"), "Missing Containment flag");
    assert!(output.contains("By (κατά)"), "Missing Delimiter flag");
}

#[test]
fn test_mosaic_unknown_stmt_coverage() {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);

    let source = "δοκιμή «τ» . 1 1 ἰσοῦται. τέλος."; // Valid Test Declaration
    let mut buffer = Vec::new();
    run_mosaic_inner(source, &mut buffer).unwrap();
    let output = String::from_utf8(buffer).unwrap();

    assert!(output.contains("Test Declaration"));
}
