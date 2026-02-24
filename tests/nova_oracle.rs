#![cfg(feature = "nova")]

use glossa::tools::oracle::run_oracle_inner;

fn run_test(source: &str) -> String {
    let mut buffer = Vec::new();
    run_oracle_inner(source, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[test]
fn test_oracle_clean() {
    let source = "
        εἶδος Χ ὁρίζειν { χ ἀριθμοῦ. }.
        ξ πέντε ἔστω.
        ξ λέγε.
    ";
    let output = run_test(source);
    assert!(output.contains("The Oracle is silent"));
}

#[test]
fn test_oracle_hubris() {
    // Deep nesting
    let source = "
        εἰ ἀληθές, {
            εἰ ἀληθές, {
                εἰ ἀληθές, {
                    εἰ ἀληθές, {
                        «Hubris» λέγε.
                    }.
                }.
            }.
        }.
    ";
    let output = run_test(source);
    assert!(output.contains("Hubris"));
    assert!(output.contains("Deep nesting detected"));
}

#[test]
fn test_oracle_laziness() {
    // Empty if block
    let source = "
        εἰ ἀληθές, { }.
    ";
    let output = run_test(source);
    assert!(output.contains("Laziness"));
    assert!(output.contains("Empty 'If' block"));
}

#[test]
fn test_oracle_laziness_while() {
    // Empty while loop
    let source = "
        ἕως ἀληθές, { }.
    ";
    let output = run_test(source);
    assert!(output.contains("Laziness"));
    assert!(output.contains("Empty 'While' loop"));
}

#[test]
fn test_oracle_barbarism() {
    // Latin type name
    let source = "
        εἶδος LatinStruct ὁρίζειν { }.
    ";
    let output = run_test(source);
    assert!(output.contains("Barbarism"));
    assert!(output.contains("latinstruct"));
}

#[test]
fn test_oracle_narcissus() {
    // Unused variable
    let source = "
        ξ πέντε ἔστω.
        «Hello» λέγε.
    ";
    let output = run_test(source);
    assert!(output.contains("Narcissus"));
    assert!(output.contains("Variable 'ξ' is declared but never used"));
}

#[test]
fn test_oracle_no_narcissus_when_used() {
    // Used variable
    let source = "
        ξ πέντε ἔστω.
        ξ λέγε.
    ";
    let output = run_test(source);
    assert!(!output.contains("Narcissus"));
}
