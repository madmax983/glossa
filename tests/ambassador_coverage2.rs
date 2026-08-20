#![cfg(feature = "nova")]

use glossa::tools::ambassador::run_ambassador;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ambassador_more_types() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test2.gl");
    let output_path = dir.path().join("out2.d.ts");

    // We can just call it via CLI
    let source = "
        εἶδος Point ὁρίζειν {
            set συνολον.
            map χαρτης.
            opt εὑρεθείη.
            res ἀποτέλεσμα.
            func ἔργον.
            unit οὐδέν.
            unk ἄγνωστον.
        }.
    ";
    fs::write(&input_path, source).unwrap();

    let res = run_ambassador(&input_path, Some(&output_path));
}
